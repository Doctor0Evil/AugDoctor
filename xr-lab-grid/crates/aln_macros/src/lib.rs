use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, AttributeArgs, ItemEnum, Lit, Meta, NestedMeta};

/// #[alngrammar(session = "SESSION", intent = "INTENT", safety = "SAFETY", evidence = "EVIDENCE")]
/// applied to an enum like `enum BciChatCommand { ... }`
///
/// Generates:
/// - impl TryFrom<&str> for Enum
/// - a `const GRAMMAR_REGEXES: [&'static str; 4]`
/// Fails compilation if any regex is empty or duplicated.
#[proc_macro_attribute]
pub fn alngrammar(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let mut session = None;
    let mut intent = None;
    let mut safety = None;
    let mut evidence = None;

    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = arg {
            let key = nv.path.get_ident().map(|i| i.to_string());
            if let (Some(k), Lit::Str(v)) = (key, nv.lit) {
                match k.as_str() {
                    "session" => session = Some(v.value()),
                    "intent" => intent = Some(v.value()),
                    "safety" => safety = Some(v.value()),
                    "evidence" => evidence = Some(v.value()),
                    _ => {}
                }
            }
        }
    }

    let session = session.expect("alngrammar: missing `session` pattern");
    let intent = intent.expect("alngrammar: missing `intent` pattern");
    let safety = safety.expect("alngrammar: missing `safety` pattern");
    let evidence = evidence.expect("alngrammar: missing `evidence` pattern");

    // Structural invariants: non-empty and all distinct.
    let patterns = [&session, &intent, &safety, &evidence];
    for p in &patterns {
        assert!(!p.is_empty(), "alngrammar: empty pattern forbidden");
    }
    for i in 0..patterns.len() {
        for j in (i + 1)..patterns.len() {
            assert!(
                patterns[i] != patterns[j],
                "alngrammar: duplicate grammar pattern detected"
            );
        }
    }

    let input = parse_macro_input!(item as ItemEnum);
    let enum_ident = &input.ident;

    // We treat a very simple grammar: first token identifies the variant.
    // Real regex compilation can be added via `regex` crate if desired.
    let mut arms = Vec::new();
    for variant in &input.variants {
        let v_ident = &variant.ident;
        let name = v_ident.to_string().to_uppercase();
        arms.push(quote! {
            s if s.eq_ignore_ascii_case(#name) => Ok(#enum_ident::#v_ident),
        });
    }

    let gen = quote! {
        #input

        impl ::core::convert::TryFrom<&str> for #enum_ident {
            type Error = &'static str;

            fn try_from(line: &str) -> Result<Self, Self::Error> {
                let trimmed = line.trim();
                match trimmed {
                    #(#arms)*
                    _ => Err("unrecognized BCI chat command"),
                }
            }
        }

        impl #enum_ident {
            pub const GRAMMAR_REGEXES: [&'static str; 4] = [
                #session,
                #intent,
                #safety,
                #evidence,
            ];
        }
    };

    gen.into()
}
