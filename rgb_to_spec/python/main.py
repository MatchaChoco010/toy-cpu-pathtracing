import pathlib
import time

import colour
import torch

FIRST_EPOCHS = 15000
FIRST_LR = 0.001
GREEN_LOSS_SCALE = 5

SECOND_EPOCHS = 15000
SECOND_LR = 0.001

TABLE_SIZE = 64
DEVICE = "cuda"
DTYPE = torch.float32
SCALE_A, SCALE_B, SCALE_C = 160.0, 35.0, 15.0


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
    def decode(raw):
        a = SCALE_A * torch.tanh(raw[..., 0])
        b = SCALE_B * raw[..., 1]
        c = SCALE_C * raw[..., 2]
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

    def rgb_from_coeff(c): return torch.matmul(spectrum_xyz(c), m_xyz2rgb.T)

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
    ).to(DEVICE).to(DTYPE)
    opt = torch.optim.Adam(mlp.parameters(), lr=FIRST_LR)

    for i in range(FIRST_EPOCHS):
        input_rgb = torch.rand((65536, 3), device=DEVICE, dtype=DTYPE)
        input_green = torch.stack([
            torch.zeros_like(input_rgb[:, 0]),
            torch.rand_like(input_rgb[:, 1]),
            torch.zeros_like(input_rgb[:, 2])
        ], dim=-1).to(DEVICE, dtype=DTYPE)

        mlp.zero_grad(set_to_none=True)
        pred_rgb = rgb_from_coeff(decode(mlp(input_rgb)))
        pred_green = rgb_from_coeff(decode(mlp(input_green)))
        loss = (pred_rgb - input_rgb).pow(2).mean() + GREEN_LOSS_SCALE * (pred_green - input_green).pow(2).mean()
        loss.backward()
        opt.step()

        delta = delta_e(pred_rgb, input_rgb, m_rgb2xyz, white_xyz)
        delta_green = delta_e(pred_green, input_green, m_rgb2xyz, white_xyz)

        if i % 1000 == 0 or i == 1 or i == FIRST_EPOCHS:
            print(f"[{cs_name}] MLP epoch {i:5d}/{FIRST_EPOCHS}  loss={loss.item():.8f}, ΔE_mean={delta.mean().item():.4f}, ΔE_green_mean={delta_green.mean().item():.4f}, ΔE_green_max={delta_green.max().item():.4f}")

    # --- 最適化 --------------------------------
    coeff_raw = torch.nn.Parameter(mlp(rgb_target))
    opt = torch.optim.Adam([coeff_raw], lr=SECOND_LR)

    for epoch in range(1, SECOND_EPOCHS + 1):
        opt.zero_grad(set_to_none=True)
        rgb_pred = rgb_from_coeff(decode(coeff_raw))
        loss = (rgb_pred - rgb_target).pow(2).mean()
        loss.backward()
        opt.step()

        delta = delta_e(rgb_pred, rgb_target, m_rgb2xyz, white_xyz)

        if epoch % 1000 == 0 or epoch == 1 or epoch == SECOND_EPOCHS:
            print(f"[{cs_name}] epoch {epoch:5d}/{SECOND_EPOCHS}  ΔE_mean={delta.mean().item():.4f}, ΔE_max={delta.max().item():.4f}")

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
