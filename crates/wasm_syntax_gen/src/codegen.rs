use crate::ast::{BoundSymbol, BuiltinSymbol, Grammar, Literal, NonTerminal, Production, Symbol};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn codegen(grammar: &Grammar) -> TokenStream {
    let mut impls: Vec<TokenStream> = Vec::with_capacity(grammar.non_terminals.len() * 2);

    for NonTerminal { name, productions } in &grammar.non_terminals {
        assert!(!productions.is_empty());

        // Generate the type (enum or struct)
        let (encoder_expr, decoder_expr) = if productions.len() > 1 {
            // enum
            let mut enum_alts: Vec<TokenStream> = Vec::with_capacity(productions.len());
            for production in productions {
                let alt_name = &production.rhs;
                let fields = enum_fields(&production.symbols);
                enum_alts.push(quote!(#alt_name(#fields)));
            }

            impls.push(quote!(
                #[derive(Debug, PartialEq)]
                pub enum #name {
                    #(#enum_alts,)*
                }
            ));

            (
                generate_enum_encoder(name, productions),
                generate_enum_decoder(grammar, name, productions),
            )
        } else {
            // struct
            let production = &productions[0];
            let fields = struct_fields(&production.symbols);

            impls.push(quote!(
                #[derive(Debug, PartialEq)]
                pub struct #name(#fields);
            ));

            let struct_name = &production.rhs;
            let field_names = production.field_names();
            let struct_value = quote!(#struct_name(#(#field_names),*));

            (
                generate_struct_encoder(name, &productions[0]),
                generate_struct_decoder(&productions[0], struct_value),
            )
        };

        impls.push(quote!(
            impl Encode for #name {
                fn encode(&self, buffer: &mut Vec<u8>) {
                    #encoder_expr
                }
            }
        ));

        impls.push(quote!(
            impl Decode for #name {
                fn decode(mut buffer: &[u8]) -> DecodeResult<#name> {
                    #decoder_expr
                }
            }
        ));
    }

    quote!(#(#impls)*)
}

fn literal_to_token_stream(lit: &Literal) -> TokenStream {
    match lit {
        Literal::U8(i) => i.to_token_stream(),
        Literal::U32(i) => i.to_token_stream(),
        Literal::I32(i) => i.to_token_stream(),
        Literal::U64(i) => i.to_token_stream(),
        Literal::I64(i) => i.to_token_stream(),
    }
}

fn bound_symbol_type(bound_symbol: &BoundSymbol) -> TokenStream {
    match bound_symbol {
        BoundSymbol::Vec(ty) => quote!(Vec<#ty>),
        BoundSymbol::Repeated(ty) => quote!(Repeated<#ty>),
        BoundSymbol::Sized(ty) => {
            let nested_ty = bound_symbol_type(ty);
            quote!(Sized<#nested_ty>)
        }
        BoundSymbol::Builtin(builtin) => match builtin {
            BuiltinSymbol::Name => quote!(Name),
            BuiltinSymbol::U32 => quote!(u32),
        },
        BoundSymbol::Type(ty) => quote!(#ty),
    }
}

impl Grammar {
    fn production_prefix(&self, production: &Production, prefix: &mut Vec<u8>) {
        assert!(!production.symbols.is_empty());

        for symbol in &production.symbols {
            match symbol {
                Symbol::Literal(lit) => prefix.extend_from_slice(&lit.as_u8s()),
                Symbol::Bound(_, _) => break,
                /*
                Symbol::Bound(_, bound_symbol) => match bound_symbol {
                    BoundSymbol::Type(non_terminal) => {
                        let non_terminal = self
                            .non_terminals
                            .iter()
                            .find(|nt| &nt.name == non_terminal)
                            .unwrap();

                        if non_terminal.productions.len() == 1 {
                            self.production_prefix(&non_terminal.productions[0], prefix);
                        }
                    }
                    BoundSymbol::Vec(_)
                    | BoundSymbol::Repeated(_)
                    | BoundSymbol::Sized(_)
                    | BoundSymbol::Builtin(_) => break,
                },
                */
            }
        }
    }
}

fn enum_fields(symbols: &[Symbol]) -> TokenStream {
    let mut fields: Vec<TokenStream> = Vec::with_capacity(symbols.len());
    for symbol in symbols {
        match symbol {
            Symbol::Literal(_) => {}
            Symbol::Bound(_, ty) => fields.push(bound_symbol_type(ty)),
        }
    }
    quote!(#(#fields,)*)
}

fn struct_fields(symbols: &[Symbol]) -> TokenStream {
    let mut fields: Vec<TokenStream> = Vec::with_capacity(symbols.len());
    for symbol in symbols {
        match symbol {
            Symbol::Literal(_) => {}
            Symbol::Bound(_, ty) => fields.push(bound_symbol_type(ty)),
        }
    }
    quote!(#(pub #fields,)*)
}

fn generate_enum_encoder(type_name: &syn::Ident, productions: &[Production]) -> TokenStream {
    let mut alts = Vec::with_capacity(productions.len());

    for production in productions {
        let mut instructions = Vec::with_capacity(production.symbols.len());
        let mut fields = Vec::with_capacity(production.symbols.len());

        for symbol in &production.symbols {
            match symbol {
                Symbol::Literal(lit) => {
                    let lit_tokens = literal_to_token_stream(lit);
                    instructions.push(quote!(#lit_tokens.encode(buffer)));
                }
                Symbol::Bound(field_name, _) => {
                    fields.push(field_name.into_token_stream());
                    instructions.push(quote!(#field_name.encode(buffer)));
                }
            }
        }

        let alt_name = &production.rhs;

        alts.push(quote!(
            #type_name::#alt_name(#(#fields,)*) => {
                #(#instructions;)*
            }
        ));
    }

    quote!(
        match self {
            #(#alts)*
        }
    )
}

fn generate_struct_encoder(type_name: &syn::Ident, production: &Production) -> TokenStream {
    let mut instructions = Vec::with_capacity(production.symbols.len());
    let mut fields = Vec::with_capacity(production.symbols.len());

    for symbol in &production.symbols {
        match symbol {
            Symbol::Literal(lit) => match lit {
                Literal::U8(u8) => instructions.push(quote!(buffer.push(#u8))),
                Literal::U32(u32) => instructions.push(quote!(#u32.encode(buffer))),
                Literal::I32(i32) => instructions.push(quote!(#i32.encode(buffer))),
                Literal::U64(u64) => instructions.push(quote!(#u64.encode(buffer))),
                Literal::I64(i64) => instructions.push(quote!(#i64.encode(buffer))),
            },
            Symbol::Bound(field_name, _) => {
                fields.push(field_name.into_token_stream());
                instructions.push(quote!(#field_name.encode(buffer)));
            }
        }
    }

    quote!(
        let #type_name(#(#fields,)*) = self;
        #(#instructions;)*
    )
}

fn generate_struct_decoder(production: &Production, value: TokenStream) -> TokenStream {
    let mut instructions: Vec<TokenStream> = Vec::with_capacity(production.symbols.len());
    let mut field_names: Vec<syn::Ident> = Vec::with_capacity(production.symbols.len());

    for symbol in &production.symbols {
        if let Symbol::Bound(name, _) = symbol {
            field_names.push(name.clone());
        }

        instructions.push(generate_symbol_decode_instructions(symbol));
    }

    quote!(
        #(#instructions)*
        Ok((#value, buffer))
    )
}

fn generate_symbol_decode_instructions(symbol: &Symbol) -> TokenStream {
    match symbol {
        Symbol::Literal(lit) => {
            let ty = match lit {
                Literal::U8(_) => quote!(u8),
                Literal::U32(_) => quote!(u32),
                Literal::I32(_) => quote!(i32),
                Literal::U64(_) => quote!(u64),
                Literal::I64(_) => quote!(i64),
            };
            let lit_tokens = literal_to_token_stream(lit);
            quote!(
                let (lit, buffer_) = #ty::decode(buffer)?;
                if lit != #lit_tokens {
                    return Err(DecodeError::Error);
                }
                buffer = buffer_;
            )
        }
        Symbol::Bound(name, bound_symbol) => match bound_symbol {
            BoundSymbol::Vec(ty) => quote!(
                let (#name, buffer_) = Vec::<#ty>::decode(buffer)?;
                buffer = buffer_;
            ),
            BoundSymbol::Repeated(ty) => quote!(
                let (#name, buffer_) = Repeated::<#ty>::decode(buffer)?;
                buffer = buffer_;
            ),
            BoundSymbol::Sized(ty) => {
                let sized_ty = bound_symbol_type(ty);
                quote!(
                    let (#name, buffer_) = Sized::<#sized_ty>::decode(buffer)?;
                    buffer = buffer_;
                )
            }
            BoundSymbol::Builtin(builtin) => match builtin {
                BuiltinSymbol::Name => quote!(
                    let (#name, buffer_) = Name::decode(buffer)?;
                    buffer = buffer_;
                ),
                BuiltinSymbol::U32 => quote!(
                    let (#name, buffer_) = u32::decode(buffer)?;
                    buffer = buffer_;
                ),
            },
            BoundSymbol::Type(ty) => quote!(
                let (#name, buffer_) = #ty::decode(buffer)?;
                buffer = buffer_;
            ),
        },
    }
}

fn generate_enum_decoder(
    grammar: &Grammar,
    type_name: &syn::Ident,
    productions: &[Production],
) -> TokenStream {
    // TODO: Make sure all alternatives start with a sequence of literals

    let mut alts: Vec<TokenStream> = Vec::with_capacity(productions.len());

    for production in productions {
        let mut firsts: Vec<u8> = Vec::with_capacity(10);
        grammar.production_prefix(production, &mut firsts);

        let pattern = quote!([#(#firsts,)*..]);

        let variant_name = &production.rhs;
        let field_names = production.field_names();
        let enum_value = quote!(#type_name::#variant_name(#(#field_names),*));

        let n_matched_symbols = firsts.len();

        let instructions = production
            .symbols
            .iter()
            .skip(n_matched_symbols)
            .map(generate_symbol_decode_instructions);

        alts.push(quote!(
            #pattern => {
                buffer = &buffer[#n_matched_symbols..];
                #(#instructions)*
                Ok((#enum_value, buffer))
            }
        ));
    }

    alts.push(quote!(
        _ => Err(DecodeError::Error)
    ));

    quote!(
        match buffer {
            #(#alts)*
        }
    )
}
