use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemEnum, ItemFn, Result, Signature, Token, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input, {Pat, PatIdent},
};

struct MethodList {
    methods: Vec<(Visibility, Signature)>,
}
impl Parse for MethodList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut methods = Vec::new();
        while !input.is_empty() {
            let vis: Visibility = input.parse()?; // pub / pub(crate) / (nothing)
            let sig: Signature = input.parse()?;
            let _comma: Option<Token![,]> = input.parse()?; // optional trailing comma
            methods.push((vis, sig));
        }
        Ok(MethodList { methods })
    }
}

/// enumのすべてのVariantの中の構造体に同じシグネチャのメソッドがあったとき、
/// そのメソッドを指定してenum自体にメソッドを定義するマクロ。
///
/// 下記の例ではVariant1とVariant2の構造体にfoo, bar, bazメソッドがあると仮定して、
/// enum MyEnumにfoo, bar, bazメソッドを定義する。
/// ```
/// # use util_macros::enum_methods;
/// struct Variant1;
/// impl Variant1 {
///    pub fn foo(&self, a: i32) -> i32 { a }
///    pub fn bar(&mut self, b: f32) -> f32 { b }
///    pub fn baz(&self) -> f32 { 0.0 }
/// }
///
/// struct Variant2;
/// impl Variant2 {
///    pub fn foo(&self, a: i32) -> i32 { a }
///    pub fn bar(&mut self, b: f32) -> f32 { b }
///    pub fn baz(&self) -> f32 { 0.0 }
/// }
///
/// #[enum_methods(
///    pub fn foo(&self, a: i32) -> i32,
///    pub fn bar(&mut self, b: f32) -> f32,
///    pub fn baz(&self) -> f32,
/// )]
/// enum MyEnum {
///    Variant1(Variant1),
///    Variant2(Variant2),
/// }
///
/// let my_enum = MyEnum::Variant1(Variant1);
/// assert_eq!(my_enum.foo(1), 1);  // enumに対して直接メソッドを呼び出せる
/// ```
#[proc_macro_attribute]
pub fn enum_methods(attr: TokenStream, item: TokenStream) -> TokenStream {
    let MethodList { methods } = parse_macro_input!(attr as MethodList);
    let enum_input = parse_macro_input!(item as ItemEnum);

    let enum_name = &enum_input.ident;
    let generics = &enum_input.generics;
    let variants = &enum_input.variants;
    let attrs = &enum_input.attrs;
    let vis = &enum_input.vis;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let method_impls = methods.iter().map(|(visibility, sig)| {
        let fn_name = &sig.ident;
        let inputs = &sig.inputs;
        let output = &sig.output;

        let self_arg = inputs
            .iter()
            .find_map(|arg| match arg {
                syn::FnArg::Receiver(r) if r.mutability.is_some() => Some(quote! { &mut self }),
                syn::FnArg::Receiver(_) => Some(quote! { &self }),
                _ => None,
            })
            .unwrap_or(quote! {});

        let args: Vec<_> = inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pat_ty) => Some(quote! { #pat_ty }),
                _ => None,
            })
            .collect();

        let arg_names: Vec<_> = inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Typed(pat_ty) => {
                    let pat = &*pat_ty.pat;
                    Some(quote! { #pat })
                }
                _ => None,
            })
            .collect();

        let match_arms = variants.iter().map(|v| {
            let v_ident = &v.ident;
            quote! {
                #enum_name::#v_ident(inner) => inner.#fn_name(#(#arg_names),*),
            }
        });

        quote! {
            #visibility fn #fn_name(#self_arg #(, #args)*) #output {
                match self {
                    #(#match_arms)*
                }
            }
        }
    });

    let output = quote! {
        #(#attrs)*
        #vis enum #enum_name #generics {
            #variants
        }

        impl #impl_generics #enum_name #ty_generics #where_clause {
            #(#method_impls)*
        }
    };

    output.into()
}

/// 二項演算子の各種参照の組み合わせの実装を自動生成するマクロ。
///
/// 次のようにして利用する。
/// ```
/// # use util_macros::impl_binary_ops;
/// # use std::marker::PhantomData;
/// # struct A<T, U>(PhantomData<T>, PhantomData<U>);
/// # struct B<T, U>(PhantomData<T>, PhantomData<U>);
/// # struct C<T, U>(PhantomData<T>, PhantomData<U>);
/// #[impl_binary_ops(Add)]
/// fn add<T, U, V>(lhs: &A<T, U>, rhs: &B<U, V>) -> C<T, V> {
///    // ...
/// #   C(PhantomData, PhantomData)
/// }
/// ```
///
/// 上記によって、`A<T, U>`と`B<U, V>`、`&A<T, U>`と`B<U, V>`、
/// `A<T, U>`と`&B<U, V>`、`&A<T, U>`と`&B<U, V>`の組み合わせの全てに二項演算が定義される。
#[proc_macro_attribute]
pub fn impl_binary_ops(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_name = parse_macro_input!(attr as syn::Path);
    let func = parse_macro_input!(item as ItemFn);

    let fn_name = &func.sig.ident;
    let fn_block = &func.block;
    let generics = &func.sig.generics;

    let mut inputs = func.sig.inputs.iter();

    // 左辺と右辺の識別子と型
    let lhs = inputs.next().expect("Expected two arguments");
    let rhs = inputs.next().expect("Expected two arguments");

    let (lhs_ident, lhs_ty) = match lhs {
        syn::FnArg::Typed(pat) => {
            let ident = match &*pat.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => panic!("lhs must be identifier"),
            };
            (ident, &pat.ty)
        }
        _ => panic!("lhs must be typed argument"),
    };

    let (rhs_ident, rhs_ty) = match rhs {
        syn::FnArg::Typed(pat) => {
            let ident = match &*pat.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => panic!("rhs must be identifier"),
            };
            (ident, &pat.ty)
        }
        _ => panic!("rhs must be typed argument"),
    };

    let output_ty = match &func.sig.output {
        syn::ReturnType::Type(_, ty) => ty,
        _ => panic!("Expected return type"),
    };

    // &TからTを抽出する
    let extract_inner_type = |ty: &Box<syn::Type>| -> Box<syn::Type> {
        if let syn::Type::Reference(syn::TypeReference { elem, .. }) = &**ty {
            elem.to_owned()
        } else {
            panic!("lhs must be &T");
        }
    };
    let lhs_ty = extract_inner_type(lhs_ty);
    let rhs_ty = extract_inner_type(rhs_ty);

    // 本体をself/rhs識別子にバインドして実行
    let replaced_block = quote! {{
        let #lhs_ident = &*self;
        let #rhs_ident = &*rhs;
        #fn_block
    }};

    let expanded = quote! {
        impl #generics std::ops::#trait_name<#rhs_ty> for #lhs_ty {
            type Output = #output_ty;
            #[inline(always)]
            fn #fn_name(self, rhs: #rhs_ty) -> Self::Output {
                (&self).#fn_name(&rhs)
            }
        }

        impl #generics std::ops::#trait_name<&#rhs_ty> for #lhs_ty {
            type Output = #output_ty;
            #[inline(always)]
            fn #fn_name(self, rhs: &#rhs_ty) -> Self::Output {
                (&self).#fn_name(rhs)
            }
        }

        impl #generics std::ops::#trait_name<#rhs_ty> for &#lhs_ty {
            type Output = #output_ty;
            #[inline(always)]
            fn #fn_name(self, rhs: #rhs_ty) -> Self::Output {
                self.#fn_name(&rhs)
            }
        }

        impl #generics std::ops::#trait_name<&#rhs_ty> for &#lhs_ty {
            type Output = #output_ty;
            #[inline(always)]
            fn #fn_name(self, rhs: &#rhs_ty) -> Self::Output #replaced_block
        }
    };

    TokenStream::from(expanded)
}

/// 二項演算子の代入演算子の各種参照の組み合わせの実装を自動生成するマクロ。
///
/// 次のようにして利用する。
/// ```
/// # use util_macros::impl_assign_ops;
/// # use std::marker::PhantomData;
/// # struct A<T, U>(PhantomData<T>, PhantomData<U>);
/// # struct B<T, U>(PhantomData<T>, PhantomData<U>);
/// #[impl_assign_ops(AddAssign)]
/// fn add_assign<T, U, V>(lhs: &mut A<T, U>, rhs: &B<U, V>){
///    // ...
/// }
/// ```
///
/// 上記によって、`&mut A<T, U>`と`B<U, V>`、`&mut A<T, U>`と`&B<U, V>`の組み合わせの両方に
/// 代入演算が定義される。
#[proc_macro_attribute]
pub fn impl_assign_ops(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_name = parse_macro_input!(attr as syn::Path);
    let func = parse_macro_input!(item as ItemFn);

    let fn_name = &func.sig.ident;
    let fn_block = &func.block;
    let generics = &func.sig.generics;

    let mut inputs = func.sig.inputs.iter();

    let lhs = inputs.next().expect("Expected two arguments");
    let rhs = inputs.next().expect("Expected two arguments");

    let (lhs_ident, lhs_ty) = match lhs {
        syn::FnArg::Typed(pat) => {
            let ident = match &*pat.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => panic!("lhs must be identifier"),
            };
            (ident, &pat.ty)
        }
        _ => panic!("lhs must be typed argument"),
    };

    let (rhs_ident, rhs_ty) = match rhs {
        syn::FnArg::Typed(pat) => {
            let ident = match &*pat.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => panic!("rhs must be identifier"),
            };
            (ident, &pat.ty)
        }
        _ => panic!("rhs must be typed argument"),
    };

    // &mut T の型から T を抽出
    let extract_inner_mut_type = |ty: &Box<syn::Type>| -> Box<syn::Type> {
        if let syn::Type::Reference(syn::TypeReference {
            mutability: Some(_),
            elem,
            ..
        }) = &**ty
        {
            elem.to_owned()
        } else {
            panic!("lhs must be &mut T");
        }
    };
    let lhs_ty = extract_inner_mut_type(lhs_ty);

    let extract_inner_ref_type = |ty: &Box<syn::Type>| -> Box<syn::Type> {
        if let syn::Type::Reference(syn::TypeReference { elem, .. }) = &**ty {
            elem.to_owned()
        } else {
            panic!("rhs must be &T");
        }
    };
    let rhs_ty = extract_inner_ref_type(rhs_ty);

    let replaced_block = quote! {{
        let #lhs_ident = &mut *self;
        let #rhs_ident = &*rhs;
        #fn_block
    }};

    let expanded = quote! {
        impl #generics std::ops::#trait_name<#rhs_ty> for #lhs_ty {
            #[inline(always)]
            fn #fn_name(&mut self, rhs: #rhs_ty) {
                self.#fn_name(&rhs)
            }
        }

        impl #generics std::ops::#trait_name<&#rhs_ty> for #lhs_ty {
            #[inline(always)]
            fn #fn_name(&mut self, rhs: &#rhs_ty) #replaced_block
        }

    };

    TokenStream::from(expanded)
}
