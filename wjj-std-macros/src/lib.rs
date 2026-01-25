use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

#[proc_macro_derive(fmt_err, attributes(err_code_prefix, error))]
pub fn derive_fmt_err(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let ctx = FmtErrContext {
        match_arms: Vec::new(),
        err_code_registrations: Vec::new(),
        template_registrations: Vec::new(),
    };
    expand(ast, "fmt_err", ctx, FmtVariant::new).into()
}

#[proc_macro_derive(raw_err, attributes(err_code_prefix, error))]
pub fn derive_raw_err(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let ctx = RawErrContext {
        match_arms: Vec::new(),
        err_code_registrations: Vec::new(),
    };
    expand(ast, "raw_err", ctx, RawVariant::new).into()
}

// ========== Traits ==========

trait ErrContextBuilder<V: VariantBuilder> {
    fn push(&mut self, enum_name: &syn::Ident, variant: V);
    fn build(self, enum_name: &syn::Ident) -> proc_macro2::TokenStream;
}

trait VariantBuilder: Sized {
    fn new(var_name: syn::Ident) -> Self;
    fn update(
        &mut self,
        ident: &str,
        lit_val: String,
        span: proc_macro2::Span,
    ) -> Result<(), proc_macro2::TokenStream>;
    fn validate(&mut self, err_code_prefix: &str) -> Result<(), proc_macro2::TokenStream>;
}

/// 校验并拼接错误码
fn validate_err_code(
    raw_code: Option<String>,
    err_code_prefix: &str,
    var_name: &syn::Ident,
) -> Result<String, proc_macro2::TokenStream> {
    match raw_code {
        Some(code) => {
            if !is_numeric_with_len(&code, 5) {
                return Err(expand_err(
                    var_name.span(),
                    "err_code must be exactly 5 digits",
                ));
            }
            Ok(format!("{}{}", err_code_prefix, code))
        }
        None => Err(expand_err_span(var_name, "err_code missing")),
    }
}

// ========== FmtErr Implementation ==========

struct FmtErrContext {
    match_arms: Vec<proc_macro2::TokenStream>,
    err_code_registrations: Vec<proc_macro2::TokenStream>,
    template_registrations: Vec<proc_macro2::TokenStream>,
}

impl ErrContextBuilder<FmtVariant> for FmtErrContext {
    fn push(&mut self, enum_name: &syn::Ident, variant: FmtVariant) {
        let err_code = &variant.err_code;
        let err_tpl = variant.err_tpl.as_ref().unwrap();
        let var_name = &variant.var_name;

        self.match_arms.push(quote! {
            #enum_name::#var_name => ::wjj_std::FmtErr {
                err_code: #err_code,
                err_tpl: #err_tpl,
                err_args: args.clone(),
            }
        });

        self.err_code_registrations
            .push(build_err_code_registration(err_code));

        let tpl_reg = format_ident!("TPL_REG_{}", err_code);
        self.template_registrations.push(quote! {
            #[linkme::distributed_slice(::wjj_std::TEMPLATE_REGISTRATIONS)]
            static #tpl_reg: ::wjj_std::TemplateRegistration =
                ::wjj_std::TemplateRegistration {
                    err_code: #err_code,
                    err_tpl: #err_tpl,
                };
        });
    }

    fn build(self, enum_name: &syn::Ident) -> proc_macro2::TokenStream {
        let match_arms = &self.match_arms;
        let err_code_registrations = &self.err_code_registrations;
        let template_registrations = &self.template_registrations;
        quote! {
            impl #enum_name {
                /// Converts this error variant into a [`FmtErr`] with the given arguments.
                pub fn to_err(&self, args: serde_json::Value) -> ::wjj_std::FmtErr {
                    match self {
                        #(#match_arms),*
                    }
                }
            }
            #(#err_code_registrations)*
            #(#template_registrations)*
        }
    }
}

struct FmtVariant {
    err_code: String,
    err_tpl: Option<String>,
    var_name: syn::Ident,
}

impl VariantBuilder for FmtVariant {
    fn new(var_name: syn::Ident) -> Self {
        FmtVariant {
            err_code: String::new(),
            err_tpl: None,
            var_name,
        }
    }

    fn update(
        &mut self,
        ident: &str,
        lit_val: String,
        span: proc_macro2::Span,
    ) -> Result<(), proc_macro2::TokenStream> {
        match ident {
            "err_code" => self.err_code = lit_val,
            "err_tpl" => self.err_tpl = Some(lit_val),
            _ => {
                return Err(expand_err(
                    span,
                    &format!(
                        "unknown attribute `{}`, expected `err_code` or `err_tpl`",
                        ident
                    ),
                ));
            }
        }
        Ok(())
    }

    fn validate(&mut self, err_code_prefix: &str) -> Result<(), proc_macro2::TokenStream> {
        let raw_code = if self.err_code.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.err_code))
        };
        self.err_code = validate_err_code(raw_code, err_code_prefix, &self.var_name)?;

        if self.err_tpl.is_none() {
            return Err(expand_err_span(&self.var_name, "err_tpl missing"));
        }
        Ok(())
    }
}

// ========== RawErr Implementation ==========

struct RawErrContext {
    match_arms: Vec<proc_macro2::TokenStream>,
    err_code_registrations: Vec<proc_macro2::TokenStream>,
}

impl ErrContextBuilder<RawVariant> for RawErrContext {
    fn push(&mut self, enum_name: &syn::Ident, variant: RawVariant) {
        let err_code = &variant.err_code;
        let err_msg = variant.err_msg.as_ref().unwrap();
        let var_name = &variant.var_name;

        self.match_arms.push(quote! {
            #enum_name::#var_name => ::wjj_std::RawErr {
                err_code: #err_code,
                err_msg: #err_msg,
            }
        });

        self.err_code_registrations
            .push(build_err_code_registration(err_code));
    }

    fn build(self, enum_name: &syn::Ident) -> proc_macro2::TokenStream {
        let match_arms = &self.match_arms;
        let err_code_registrations = &self.err_code_registrations;
        quote! {
            impl #enum_name {
                /// Converts this error variant into a [`RawErr`].
                pub fn to_err(&self) -> ::wjj_std::RawErr {
                    match self {
                        #(#match_arms),*
                    }
                }
            }
            #(#err_code_registrations)*
        }
    }
}

struct RawVariant {
    err_code: String,
    err_msg: Option<String>,
    var_name: syn::Ident,
}

impl VariantBuilder for RawVariant {
    fn new(var_name: syn::Ident) -> Self {
        RawVariant {
            err_code: String::new(),
            err_msg: None,
            var_name,
        }
    }

    fn update(
        &mut self,
        ident: &str,
        lit_val: String,
        span: proc_macro2::Span,
    ) -> Result<(), proc_macro2::TokenStream> {
        match ident {
            "err_code" => self.err_code = lit_val,
            "err_msg" => self.err_msg = Some(lit_val),
            _ => {
                return Err(expand_err(
                    span,
                    &format!(
                        "unknown attribute `{}`, expected `err_code` or `err_msg`",
                        ident
                    ),
                ));
            }
        }
        Ok(())
    }

    fn validate(&mut self, err_code_prefix: &str) -> Result<(), proc_macro2::TokenStream> {
        let raw_code = if self.err_code.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.err_code))
        };
        self.err_code = validate_err_code(raw_code, err_code_prefix, &self.var_name)?;

        if self.err_msg.is_none() {
            return Err(expand_err_span(&self.var_name, "err_msg missing"));
        }
        Ok(())
    }
}

// ========== Common Functions ==========

/// 构建错误码注册的公共逻辑
fn build_err_code_registration(err_code: &str) -> proc_macro2::TokenStream {
    let err_code_reg = format_ident!("ERR_CODE_{}", err_code);
    quote! {
        #[linkme::distributed_slice(::wjj_std::ERR_CODE_REGISTRATIONS)]
        static #err_code_reg: ::wjj_std::ErrCodeRegistration =
            ::wjj_std::ErrCodeRegistration {
                err_code: #err_code,
            };
    }
}

fn expand<Ctx, V, F>(
    ast: syn::DeriveInput,
    derive_name: &str,
    mut ctx: Ctx,
    variant_new: F,
) -> proc_macro2::TokenStream
where
    V: VariantBuilder,
    Ctx: ErrContextBuilder<V>,
    F: Fn(syn::Ident) -> V,
{
    let enum_name = &ast.ident;
    let syn::Data::Enum(ref e) = ast.data else {
        return expand_err_span(&ast, &format!("{} only works on enums", derive_name));
    };
    let variants = &e.variants;

    // 解析 enum 上的 err_code_prefix
    let mut err_code_prefix: Option<String> = None;
    for attr in &ast.attrs {
        if attr.path().is_ident("error") {
            return expand_err_span(attr, "`error` attribute is only allowed on enum variants");
        }
        if attr.path().is_ident("err_code_prefix") {
            match parse_err_code_prefix(attr) {
                Ok(prefix) => err_code_prefix = Some(prefix),
                Err(e) => return e,
            }
        }
    }

    let err_code_prefix = match err_code_prefix {
        Some(prefix) => prefix,
        None => {
            return expand_err_span(
                &ast.ident,
                "`err_code_prefix` attribute is required on the enum",
            );
        }
    };

    for variant in variants {
        let var_name = &variant.ident;
        let mut v = variant_new(var_name.clone());

        for attr in &variant.attrs {
            if attr.path().is_ident("err_code_prefix") {
                return expand_err_span(
                    attr,
                    "`err_code_prefix` attribute is only allowed on the enum, not on variants",
                );
            }
            if attr.path().is_ident("error")
                && let Err(e) = parse_error_attr(attr, &mut v)
            {
                return e;
            }
        }

        let Err(e) = v.validate(&err_code_prefix) else {
            ctx.push(enum_name, v);
            continue;
        };
        return e;
    }

    ctx.build(enum_name)
}

fn parse_err_code_prefix(attr: &syn::Attribute) -> Result<String, proc_macro2::TokenStream> {
    let syn::Meta::NameValue(nv) = &attr.meta else {
        return Err(expand_err_span(
            attr,
            "`err_code_prefix` must be written as #[err_code_prefix = \"001\"]",
        ));
    };

    let syn::Expr::Lit(expr_lit) = &nv.value else {
        return Err(expand_err_span(
            &nv.value,
            "`err_code_prefix` must be a string literal like \"001\"",
        ));
    };

    let syn::Lit::Str(lit) = &expr_lit.lit else {
        return Err(expand_err_span(
            &expr_lit.lit,
            "`err_code_prefix` value must be a string literal, e.g. \"001\"",
        ));
    };

    let prefix = lit.value();
    if !is_numeric_with_len(&prefix, 3) {
        return Err(expand_err(
            lit.span(),
            "`err_code_prefix` must be exactly 3 digits, e.g. \"001\"",
        ));
    }
    Ok(prefix)
}

fn parse_error_attr<V: VariantBuilder>(
    attr: &syn::Attribute,
    variant: &mut V,
) -> Result<(), proc_macro2::TokenStream> {
    let syn::Meta::List(list) = attr.meta.clone() else {
        return Ok(());
    };

    let args = list
        .parse_args_with(
            syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated,
        )
        .map_err(|e| e.to_compile_error())?;

    for nv in args {
        let Some(i) = nv.path.get_ident() else {
            return Err(expand_err_span(
                &nv.path,
                "expected identifier like `err_code = \"00001\"`",
            ));
        };
        let ident = i.to_string();

        let syn::Expr::Lit(expr_lit) = &nv.value else {
            return Err(expand_err_span(&nv.value, "value must be literal string"));
        };

        let syn::Lit::Str(lit_str) = &expr_lit.lit else {
            return Err(expand_err_span(
                &expr_lit.lit,
                "value must be string literal",
            ));
        };

        variant.update(&ident, lit_str.value(), nv.path.span())?;
    }
    Ok(())
}

/// 校验字符串是否为指定长度的纯数字
fn is_numeric_with_len(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_digit())
}

/// 结构错误 - 用于 AST 节点类型不符、属性放错位置等
fn expand_err_span(token: impl quote::ToTokens, msg: &str) -> proc_macro2::TokenStream {
    syn::Error::new_spanned(token, msg).to_compile_error()
}

/// 值错误 - 用于字符串格式不对、长度不对等
fn expand_err(span: proc_macro2::Span, msg: &str) -> proc_macro2::TokenStream {
    syn::Error::new(span, msg).to_compile_error()
}
