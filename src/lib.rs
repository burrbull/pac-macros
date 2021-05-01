extern crate proc_macro;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

fn simple_derive(
    input: TokenStream,
    f: impl FnOnce(Ident, TokenStream) -> TokenStream,
) -> TokenStream {
    let mut tt = input.into_iter();
    loop {
        match tt.next() {
            Some(TokenTree::Ident(pb)) if pb.to_string() == "pub" => break,
            None => panic!("pub not found"),
            _ => {}
        }
    }
    match (tt.next(), tt.next(), tt.next()) {
        (
            Some(TokenTree::Ident(strct)),
            Some(TokenTree::Ident(name)),
            Some(TokenTree::Group(group)),
        ) if strct.to_string() == "struct" && group.delimiter() == Delimiter::Parenthesis => {
            f(name, group.stream())
        }
        _ => panic!("Can't parse structure"),
    }
}

#[proc_macro_derive(From)]
pub fn derive_from(input: TokenStream) -> TokenStream {
    simple_derive(input, derive_from_)
}

#[proc_macro_derive(Deref)]
pub fn derive_deref(input: TokenStream) -> TokenStream {
    simple_derive(input, |name, ty_generics| {
        derive_deref_or_deref_mut(name, ty_generics, false)
    })
}

#[proc_macro_derive(DerefMut)]
pub fn derive_deref_mut(input: TokenStream) -> TokenStream {
    simple_derive(input, |name, ty_generics| {
        derive_deref_or_deref_mut(name, ty_generics, true)
    })
}

fn derive_from_(name: Ident, ty_generics: TokenStream) -> TokenStream {
    let v: Vec<TokenTree> = vec![
        Ident::new("impl", Span::call_site()).into(),
        Ident::new("From", Span::call_site()).into(),
        Punct::new('<', Spacing::Alone).into(),
    ];
    let mut s = v.into_iter().collect::<TokenStream>();
    s.extend(ty_generics.clone());
    let v: Vec<TokenTree> = vec![
        Punct::new('>', Spacing::Alone).into(),
        Ident::new("for", Span::call_site()).into(),
        name.clone().into(),
        Group::new(Delimiter::Brace, {
            let mut s = inline();
            let v: Vec<TokenTree> = vec![
                Ident::new("fn", Span::call_site()).into(),
                Ident::new("from", Span::call_site()).into(),
                Group::new(Delimiter::Parenthesis, {
                    let v: Vec<TokenTree> = vec![
                        Ident::new("inner", Span::call_site()).into(),
                        Punct::new(':', Spacing::Alone).into(),
                    ];
                    let mut s = v.into_iter().collect::<TokenStream>();
                    s.extend(ty_generics);
                    s
                })
                .into(),
                Punct::new('-', Spacing::Joint).into(),
                Punct::new('>', Spacing::Alone).into(),
                Ident::new("Self", Span::call_site()).into(),
                Group::new(Delimiter::Brace, {
                    let v: Vec<TokenTree> = vec![
                        name.into(),
                        Group::new(
                            Delimiter::Parenthesis,
                            TokenStream::from(TokenTree::from(Ident::new(
                                "inner",
                                Span::call_site(),
                            ))),
                        )
                        .into(),
                    ];
                    v.into_iter().collect::<TokenStream>()
                })
                .into(),
            ];
            s.extend(v.into_iter());
            s
        })
        .into(),
    ];
    s.extend(v.into_iter());
    s
}
fn derive_deref_or_deref_mut(name: Ident, ty_generics: TokenStream, mt: bool) -> TokenStream {
    fn target(ty_generics: TokenStream) -> TokenStream {
        let v: Vec<TokenTree> = vec![
            Ident::new("type", Span::call_site()).into(),
            Ident::new("Target", Span::call_site()).into(),
            Punct::new('=', Spacing::Alone).into(),
        ];
        let mut s = v.into_iter().collect::<TokenStream>();
        s.extend(ty_generics);
        s.extend(TokenStream::from(TokenTree::from(Punct::new(
            ';',
            Spacing::Alone,
        ))));
        s
    }

    let add_mut = || TokenStream::from(TokenTree::from(Ident::new("mut", Span::call_site())));

    let v: Vec<TokenTree> = vec![
        Ident::new("impl", Span::call_site()).into(),
        Ident::new("core", Span::call_site()).into(),
        Punct::new(':', Spacing::Joint).into(),
        Punct::new(':', Spacing::Alone).into(),
        Ident::new("ops", Span::call_site()).into(),
        Punct::new(':', Spacing::Joint).into(),
        Punct::new(':', Spacing::Alone).into(),
    ];
    let mut s = v.into_iter().collect::<TokenStream>();
    s.extend(TokenStream::from(TokenTree::from(Ident::new(
        if mt { "DerefMut" } else { "Deref" },
        Span::call_site(),
    ))));
    let v: Vec<TokenTree> = vec![
        Ident::new("for", Span::call_site()).into(),
        name.into(),
        Group::new(Delimiter::Brace, {
            let mut s = TokenStream::new();
            if !mt {
                s.extend(target(ty_generics));
            }
            s.extend(inline());
            s.extend(TokenStream::from(TokenTree::from(Ident::new(
                "fn",
                Span::call_site(),
            ))));

            s.extend(TokenStream::from(TokenTree::from(Ident::new(
                if mt { "deref_mut" } else { "deref" },
                Span::call_site(),
            ))));

            let v: Vec<TokenTree> = vec![
                Group::new(Delimiter::Parenthesis, {
                    let mut s = TokenStream::from(TokenTree::from(Punct::new('&', Spacing::Alone)));
                    if mt {
                        s.extend(add_mut());
                    }
                    s.extend(TokenStream::from(TokenTree::from(Ident::new(
                        "self",
                        Span::call_site(),
                    ))));
                    s
                })
                .into(),
                Punct::new('-', Spacing::Joint).into(),
                Punct::new('>', Spacing::Alone).into(),
                Punct::new('&', Spacing::Alone).into(),
            ];
            s.extend(v.into_iter());
            if mt {
                s.extend(add_mut());
            }
            let v: Vec<TokenTree> = vec![
                Ident::new("Self", Span::call_site()).into(),
                Punct::new(':', Spacing::Joint).into(),
                Punct::new(':', Spacing::Alone).into(),
                Ident::new("Target", Span::call_site()).into(),
                Group::new(Delimiter::Brace, {
                    let mut s = TokenStream::from(TokenTree::from(Punct::new('&', Spacing::Alone)));
                    if mt {
                        s.extend(add_mut());
                    }
                    let v: Vec<TokenTree> = vec![
                        Ident::new("self", Span::call_site()).into(),
                        Punct::new('.', Spacing::Alone).into(),
                        Literal::usize_unsuffixed(0).into(),
                    ];
                    s.extend(v.into_iter());
                    s
                })
                .into(),
            ];
            s.extend(v.into_iter());
            s
        })
        .into(),
    ];

    s.extend(v.into_iter());
    s
}

fn inline() -> TokenStream {
    let v: Vec<TokenTree> = vec![
        Punct::new('#', Spacing::Alone).into(),
        Group::new(Delimiter::Bracket, {
            let v: Vec<TokenTree> = vec![
                Ident::new("inline", Span::call_site()).into(),
                Group::new(
                    Delimiter::Parenthesis,
                    TokenTree::from(Ident::new("always", Span::call_site())).into(),
                )
                .into(),
            ];
            v.into_iter().collect::<TokenStream>()
        })
        .into(),
    ];
    v.into_iter().collect::<TokenStream>()
}
