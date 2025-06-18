import pathlib
import time

import colour
import torch
from torch.optim.lr_scheduler import CosineAnnealingLR

FIRST_EPOCHS = 100000
FIRST_LR = 0.0001
N_POOL = 10_000_000
FIRST_BATCH = 16384
RGB_LOSS_SCALE = 1
GREEN_LOSS_SCALE = 3
DARK_LOSS_SCALE = 1

SECOND_EPOCHS = 30000
SECOND_LR = 0.001

TABLE_SIZE = 64
DEVICE = "cuda"
DTYPE = torch.float32

SCALE_A = 100.0
SCALE_B = 100.0
SCALE_C = 10.0

# -------------------------------------
# 波長軸の設定とCIEのX, Y, Zの軸、正規化d65光源を取得
# -------------------------------------
w_min, w_max, step = 360.0, 830.0, 1.0

shape = colour.SpectralShape(w_min, w_max, step)

cmfs = colour.MSDS_CMFS['CIE 1931 2 Degree Standard Observer'].copy().align(shape)

x_bar = torch.as_tensor(cmfs.values[:, 0], device=DEVICE, dtype=DTYPE)
y_bar = torch.as_tensor(cmfs.values[:, 1], device=DEVICE, dtype=DTYPE)
z_bar = torch.as_tensor(cmfs.values[:, 2], device=DEVICE, dtype=DTYPE)
wavelengths = torch.as_tensor(cmfs.wavelengths, device=DEVICE, dtype=DTYPE)
wavelength_norm   = (wavelengths - w_min) / (w_max - w_min)
wavelength_norm2  = wavelength_norm ** 2

d65 = colour.SDS_ILLUMINANTS['D65'].copy().align(shape)
d65_values = torch.as_tensor(d65.values, device=DEVICE, dtype=DTYPE)
y_d65 = (d65_values * y_bar).sum() * step
k = 1.0 / y_d65
d65_norm = d65_values * k

# -------------------------------------
# sRGBのXYZ変換行列と白色点を取得
# -------------------------------------
cs_srgb = colour.RGB_COLOURSPACES["sRGB"]
m_xyz_to_rgb = torch.as_tensor(cs_srgb.matrix_XYZ_to_RGB, device=DEVICE, dtype=DTYPE)
m_rgb_to_xyz = torch.as_tensor(cs_srgb.matrix_RGB_to_XYZ, device=DEVICE, dtype=DTYPE)

xy_white = cs_srgb.whitepoint
white_xyz_np = colour.xyY_to_XYZ((*xy_white, 1.0))
white_xyz = torch.as_tensor(white_xyz_np, device=DEVICE, dtype=DTYPE)

# -------------------------------------
# z_nodesとカラースペース定義
# -------------------------------------
def smoothstep(x):
    return x * x * (3.0 - 2.0 * x)

idx_int   = torch.arange(TABLE_SIZE, device=DEVICE, dtype=torch.long)
idx_float = idx_int.to(DTYPE)
z_nodes = smoothstep(smoothstep(idx_float / (TABLE_SIZE - 1)))

# colour-science の色域キーと出力ファイル名
SPACES = {
    "sRGB":             "../tables//srgb_table.bin",
    "P3-D65":           "../tables/dcip3d65_table.bin",
    "Adobe RGB (1998)": "../tables/adobergb_table.bin",
    "ITU-R BT.2020":    "../tables/rec2020_table.bin",
    "ACEScg":           "../tables/acescg_table.bin",
    "ACES2065-1":       "../tables/aces2065_1_table.bin",
}

# -------------------------------------
# xyz -> labの変換関数とdelta Eの計算関数
# -------------------------------------
delta  = 6.0 / 29.0
d_sq   = delta ** 2
d_cb   = delta ** 3

def signed_cbrt(t): return torch.sign(t) * torch.pow(torch.abs(t), 1.0 / 3.0)

def xyz_to_lab(xyz, white_xyz):
    xyz_scaled = xyz / white_xyz
    def f(t): return torch.where(t > d_cb, signed_cbrt(t), t / (3 * d_sq) + 4/29)
    fx, fy, fz = (f(xyz_scaled[..., i]) for i in range(3))
    L = 116*fy - 16
    a = 500*(fx - fy)
    b = 200*(fy - fz)
    return torch.stack([L, a, b], dim=-1)

def delta_e(rgb_pred, rgb_ref, m_rgb_to_xyz, white_xyz):
    xyz_p = torch.matmul(rgb_pred, m_rgb_to_xyz.T)
    xyz_r = torch.matmul(rgb_ref , m_rgb_to_xyz.T)
    return torch.linalg.norm(
        xyz_to_lab(xyz_p, white_xyz) - xyz_to_lab(xyz_r, white_xyz), dim=-1
    )

# -------------------------------------
# 乱数の事前準備
# -------------------------------------

# ランダムなRGB値を生成
rand_rgb_pool = torch.rand((N_POOL, 3), device=DEVICE, dtype=DTYPE)

# 純緑が苦手なので学習データを個別に追加
rand_green_pool = torch.stack([
    torch.zeros_like(rand_rgb_pool[:, 0]),
    torch.rand_like(rand_rgb_pool[:, 1]),
    torch.zeros_like(rand_rgb_pool[:, 2])
], dim=-1).to(DEVICE, dtype=DTYPE)

# 暗くてかつRGBの数値のいずれかが0のパターンが苦手なのでデータを個別に追加
num_zeros = torch.randint(0, 3, (N_POOL,), device=DEVICE)
rand_indices = torch.argsort(torch.rand((N_POOL, 3), device=DEVICE), dim=1)
mask = (rand_indices >= num_zeros.unsqueeze(1)).to(DTYPE)
rand_dark_pool = torch.rand((N_POOL, 3), device=DEVICE, dtype=DTYPE) * 0.3 * mask


# -------------------------------------
# 最適化ループ
# -------------------------------------
def train_space(cs_name, out_file):
    # --- カラースペース設定 -----------------------
    cs        = colour.RGB_COLOURSPACES[cs_name]
    m_xyz2rgb = torch.as_tensor(cs.matrix_XYZ_to_RGB, device=DEVICE, dtype=DTYPE)
    m_rgb2xyz = torch.as_tensor(cs.matrix_RGB_to_XYZ, device=DEVICE, dtype=DTYPE)
    xy_white  = cs.whitepoint
    white_xyz = torch.as_tensor(colour.xyY_to_XYZ((*xy_white, 1.0)),
                                device=DEVICE, dtype=DTYPE)

    # --- RGB ターゲットテーブル -------------------
    zi, yi, xi = torch.meshgrid(idx_int, idx_int, idx_int, indexing="ij")
    z = z_nodes[zi]
    y = yi.to(DTYPE) / (TABLE_SIZE - 1) * z
    x = xi.to(DTYPE) / (TABLE_SIZE - 1) * z

    rgb_target = torch.stack(
        [torch.stack([z, x, y], -1),
         torch.stack([y, z, x], -1),
         torch.stack([x, y, z], -1)],
        0
    )

    # --- 係数スケール ---------------------------
    init_scale = torch.ones((3, ), device=DEVICE, dtype=torch.float32)
    log_scale = torch.nn.Parameter(init_scale.log())

    def decode(raw):
        scale_a = log_scale.exp()[0] * SCALE_A
        scale_b = log_scale.exp()[1] * SCALE_B
        scale_c = log_scale.exp()[2] * SCALE_C
        a = scale_a * torch.tanh(raw[..., 0])
        b = scale_b * torch.tanh(raw[..., 1])
        c = scale_c * torch.tanh(raw[..., 2])
        return torch.stack([a, b, c], dim=-1)

    # --- 学習関数 --------------------------------
    def spectrum_xyz(c):
        a, b, c0 = c[..., 0], c[..., 1], c[..., 2]
        a, b, c0 = (t.unsqueeze(-1) for t in (a, b, c0))
        spec = torch.sigmoid(a * wavelength_norm2 + b * wavelength_norm + c0)
        X = (spec * x_bar * d65_norm).sum(-1) * step
        Y = (spec * y_bar * d65_norm).sum(-1) * step
        Z = (spec * z_bar * d65_norm).sum(-1) * step
        return torch.stack([X, Y, Z], -1)

    def rgb_from_coeff(c):
        rgb = torch.matmul(spectrum_xyz(c), m_xyz2rgb.T)
        rgb = torch.max(rgb, torch.zeros_like(rgb))
        return rgb

    # --- 初期値決定用NN --------------------------------
    mlp = torch.nn.Sequential(
        torch.nn.Linear(3, 512),
        torch.nn.ReLU(),
        torch.nn.Linear(512, 512),
        torch.nn.ReLU(),
        torch.nn.Linear(512, 512),
        torch.nn.ReLU(),
        torch.nn.Linear(512, 512),
        torch.nn.ReLU(),
        torch.nn.Linear(512, 512),
        torch.nn.ReLU(),
        torch.nn.Linear(512, 3)
    ).to(DEVICE).to(torch.float32)
    opt = torch.optim.Adam([
        { "params": mlp.parameters(), "lr": FIRST_LR },
        { "params": log_scale, "lr": FIRST_LR * 10 },
    ])
    scheduler = CosineAnnealingLR(opt, FIRST_EPOCHS)

    for i in range(1, FIRST_EPOCHS + 1):
        rand_idx  = torch.randint(0, N_POOL, (FIRST_BATCH,))
        input_rgb = rand_rgb_pool[rand_idx]

        rand_green_idx  = torch.randint(0, N_POOL, (FIRST_BATCH,))
        input_green = rand_green_pool[rand_green_idx]

        rand_dark_idx = torch.randint(0, N_POOL, (FIRST_BATCH,))
        input_dark = rand_dark_pool[rand_dark_idx]


        pred_rgb = rgb_from_coeff(decode(mlp(input_rgb)))
        rgb_loss = (pred_rgb - input_rgb).pow(2).mean() * RGB_LOSS_SCALE
        reg_loss = log_scale.exp().pow(2).sum() * 1e-5

        mlp.zero_grad(set_to_none=True)
        (rgb_loss + reg_loss).backward()
        opt.step()


        pred_green = rgb_from_coeff(decode(mlp(input_green)))
        green_loss = (pred_green - input_green).pow(2).mean() * GREEN_LOSS_SCALE
        reg_loss = log_scale.exp().pow(2).sum() * 1e-5

        mlp.zero_grad(set_to_none=True)
        (green_loss + reg_loss).backward()
        opt.step()


        pred_dark = rgb_from_coeff(decode(mlp(input_dark)))
        dark_loss = (pred_dark - input_dark).pow(2).mean() * DARK_LOSS_SCALE
        reg_loss = log_scale.exp().pow(2).sum() * 1e-5

        mlp.zero_grad(set_to_none=True)
        (dark_loss + reg_loss).backward()
        opt.step()


        scheduler.step()


        delta = delta_e(pred_rgb, input_rgb, m_rgb2xyz, white_xyz)
        delta_green = delta_e(pred_green, input_green, m_rgb2xyz, white_xyz)
        delta_dark = delta_e(pred_dark, input_dark, m_rgb2xyz, white_xyz)
        delta_max = torch.max(torch.cat([delta, delta_green, delta_dark]))

        loss = rgb_loss + green_loss + dark_loss + reg_loss

        if i % 1000 == 0 or i == 1 or i == FIRST_EPOCHS:
            print(f"[{cs_name}] MLP epoch {i:5d}/{FIRST_EPOCHS}  loss={loss.item():.8f}, ΔE_mean={delta.mean().item():.4f}, ΔE_green_mean={delta_green.mean().item():.4f}, ΔE_dark_mean={delta_dark.mean().item():.4f}, ΔE_max={delta_max.item():.4f}")

    # --- 最適化 --------------------------------
    coeff_raw = torch.nn.Parameter(mlp(rgb_target.to(torch.float32)))
    opt = torch.optim.Adam([coeff_raw], lr=SECOND_LR)
    scheduler = CosineAnnealingLR(opt, SECOND_EPOCHS)

    for epoch in range(1, SECOND_EPOCHS + 1):
        opt.zero_grad(set_to_none=True)

        rgb_pred = rgb_from_coeff(decode(coeff_raw))
        delta = delta_e(rgb_pred, rgb_target, m_rgb2xyz, white_xyz)
        loss = delta.mean()

        loss.backward()
        opt.step()
        scheduler.step()

        if epoch % 1000 == 0 or epoch == 1 or epoch == SECOND_EPOCHS:
            print(f"[{cs_name}] OPT epoch {epoch:5d}/{SECOND_EPOCHS}  ΔE_mean={delta.mean().item():.4f}, ΔE_max={delta.max().item():.4f}")

        if epoch % 2500 == 0:
            out_path = pathlib.Path(out_file)
            with out_path.open("wb") as f:
                z_nodes.cpu().numpy().astype("<f4").tofile(f)
                decode(coeff_raw).detach().cpu().numpy().astype("<f4").ravel().tofile(f)
            print(f"  → saved {out_path} ({out_path.stat().st_size/1_048_576:.2f} MB)")

    # --- 最終バイナリ出力 -----------------------------
    out_path = pathlib.Path(out_file)
    with out_path.open("wb") as f:
        z_nodes.cpu().numpy().astype("<f4").tofile(f)
        decode(coeff_raw).detach().cpu().numpy().astype("<f4").ravel().tofile(f)
    print(f"  → saved {out_path} ({out_path.stat().st_size/1_048_576:.2f} MB)")


# -------------------------------------
# 対応する全色域で最適化と保存を実行
# -------------------------------------
start_time = time.time()
for cs_key, filename in SPACES.items():
    cs_start_time = time.time()

    train_space(cs_key, filename)

    cs_end_time = time.time()
    elapsed_time = int(cs_end_time - cs_start_time)
    elapsed_hour = elapsed_time // 3600
    elapsed_minute = (elapsed_time % 3600) // 60
    elapsed_second = (elapsed_time % 3600 % 60)
    print(f"[{cs_key}] Processed in {elapsed_hour}h {elapsed_minute}m {elapsed_second}s.")

end_time = time.time()
elapsed_time = int(end_time - start_time)
elapsed_hour = elapsed_time // 3600
elapsed_minute = (elapsed_time % 3600) // 60
elapsed_second = (elapsed_time % 3600 % 60)
print(f"All spaces processed in {elapsed_hour}h {elapsed_minute}m {elapsed_second}s.")
