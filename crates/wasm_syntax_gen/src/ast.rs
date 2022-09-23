use syn::parse::{Parse, ParseBuffer};

#[derive(Debug)]
pub struct Grammar {
    pub non_terminals: Vec<NonTerminal>,
}

#[derive(Debug)]
pub struct NonTerminal {
    pub name: syn::Ident,

    /// Non-empty list of productions
    pub productions: Vec<Production>,
}

#[derive(Debug)]
pub struct Production {
    /// Non-empty list of symbols
    pub symbols: Vec<Symbol>,

    /// Right-hand side of a production. This identifier is used as the enum variant name in enums.
    /// For structs this is currently ignored.
    pub rhs: syn::Ident,
}

impl Production {
    pub fn field_names(&self) -> impl Iterator<Item = &syn::Ident> {
        self.symbols.iter().filter_map(|symbol| match symbol {
            Symbol::Literal(_) => None,
            Symbol::Bound(name, _) => Some(name),
        })
    }
}

#[derive(Debug)]
pub enum Symbol {
    /// A LEB-128 encoded integer literal.
    Literal(Literal),

    /// A symbol bound to a name: `a:...`
    Bound(syn::Ident, BoundSymbol),
}

#[derive(Debug, Clone, Copy)]
pub enum Literal {
    U8(u8),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
}

impl Literal {
    pub fn as_u8s(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(9);
        match self {
            Literal::U8(i) => ret.push(*i),
            Literal::U32(i) => {
                leb128::write::unsigned(&mut ret, u64::from(*i)).unwrap();
            }
            Literal::I32(i) => {
                leb128::write::signed(&mut ret, i64::from(*i)).unwrap();
            }
            Literal::U64(i) => {
                leb128::write::unsigned(&mut ret, *i).unwrap();
            }
            Literal::I64(i) => {
                leb128::write::signed(&mut ret, *i).unwrap();
            }
        }
        ret
    }
}

/// A symbol bound to a name
#[derive(Debug)]
pub enum BoundSymbol {
    /// A vector: `vec(<type>)`
    Vec(syn::Ident),

    /// Zero or more things: `repeated(<type>)`. Unlike `Vec`, encoding of this type of fields do
    /// not have length prefix.
    Repeated(syn::Ident),

    /// A sized section: `sized(<type>)`
    Sized(Box<BoundSymbol>),

    /// A built-in type: `name`, `u32` etc.
    Builtin(BuiltinSymbol),

    /// A user-written type (not built-in)
    Type(syn::Ident),
}

#[derive(Debug)]
pub enum BuiltinSymbol {
    Name,
    U32,
}

impl Parse for Grammar {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let mut non_terminals = Vec::new();

        while !input.is_empty() {
            non_terminals.push(NonTerminal::parse(input)?);
        }

        Ok(Grammar { non_terminals })
    }
}

impl Parse for NonTerminal {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let name = syn::Ident::parse(input)?;

        let mut productions = Vec::new();

        let braced;
        syn::braced!(braced in input);

        while !braced.is_empty() {
            productions.push(Production::parse(&braced)?);
        }

        if productions.is_empty() {
            return Err(input.error("Non-terminal should have at least one production"));
        }

        Ok(NonTerminal { name, productions })
    }
}

impl Parse for Production {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let mut symbols = Vec::new();

        while !input.peek(syn::token::Eq) {
            symbols.push(Symbol::parse(input)?);
        }

        if symbols.is_empty() {
            return Err(input.error("Production should have at least one symbol"));
        }

        syn::token::Eq::parse(input)?;

        let rhs = syn::Ident::parse(input)?;

        syn::token::Comma::parse(input)?;

        Ok(Production { symbols, rhs })
    }
}

impl Parse for Symbol {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        if input.peek(syn::LitInt) {
            let lit = syn::LitInt::parse(input).unwrap();

            // `0xAB` or `0xAB:u32` (or `123:u32` etc.)
            if input.peek(syn::token::Colon) {
                syn::token::Colon::parse(input).unwrap();
                let lit_ty = syn::Ident::parse(input)?.to_string();
                match lit_ty.as_str() {
                    "i32" => Ok(Symbol::Literal(Literal::I32(lit.base10_parse()?))),
                    "u32" => Ok(Symbol::Literal(Literal::U32(lit.base10_parse()?))),
                    "i64" => Ok(Symbol::Literal(Literal::I64(lit.base10_parse()?))),
                    "u64" => Ok(Symbol::Literal(Literal::U64(lit.base10_parse()?))),
                    _ => {
                        Err(input.error("Literal type can be one of: i32, u32, i64, u64, f32, f64"))
                    }
                }
            } else {
                let lit_value = lit.base10_parse::<u8>()?;
                Ok(Symbol::Literal(Literal::U8(lit_value)))
            }
        } else if input.peek(syn::Ident) {
            let ident = syn::Ident::parse(input).unwrap();
            syn::token::Colon::parse(input)?;
            let symbol = BoundSymbol::parse(input).unwrap();
            Ok(Symbol::Bound(ident, symbol))
        } else {
            Err(input.error("Expected a u8 literal or identifier"))
        }
    }
}

impl Parse for BoundSymbol {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let ident = syn::Ident::parse(input)?;
        match ident.to_string().as_str() {
            "vec" => {
                let parenthesized;
                syn::parenthesized!(parenthesized in input);
                let type_name = syn::Ident::parse(&parenthesized)?;
                if !parenthesized.is_empty() {
                    return Err(parenthesized.error("Expected type name in `vec(...)`"));
                }
                Ok(BoundSymbol::Vec(type_name))
            }

            "repeated" => {
                let parenthesized;
                syn::parenthesized!(parenthesized in input);
                let type_name = syn::Ident::parse(&parenthesized)?;
                if !parenthesized.is_empty() {
                    return Err(parenthesized.error("Expected type name in `repeated(...)`"));
                }
                Ok(BoundSymbol::Repeated(type_name))
            }

            "sized" => {
                let parenthesized;
                syn::parenthesized!(parenthesized in input);
                let sized_thing = BoundSymbol::parse(&parenthesized)?;
                if !parenthesized.is_empty() {
                    return Err(parenthesized.error("Expected type in `sized(...)`"));
                }
                Ok(BoundSymbol::Sized(Box::new(sized_thing)))
            }

            "name" => Ok(BoundSymbol::Builtin(BuiltinSymbol::Name)),

            "u32" => Ok(BoundSymbol::Builtin(BuiltinSymbol::U32)),

            _ => Ok(BoundSymbol::Type(ident)),
        }
    }
}

#[test]
fn parse_bound_symbol_vec() {
    match syn::parse_str::<BoundSymbol>("vec(A)").unwrap() {
        BoundSymbol::Vec(ident) => assert_eq!(ident.to_string(), "A"),
        _ => panic!(),
    }
}

#[test]
fn parse_bound_symbol_sized() {
    match syn::parse_str::<BoundSymbol>("sized(A)").unwrap() {
        BoundSymbol::Sized(sized) => match *sized {
            BoundSymbol::Type(ident) => assert_eq!(ident.to_string(), "A"),
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn parse_bound_symbol_builtin() {
    match syn::parse_str::<BoundSymbol>("u32").unwrap() {
        BoundSymbol::Builtin(BuiltinSymbol::U32) => (),
        _ => panic!(),
    }

    match syn::parse_str::<BoundSymbol>("name").unwrap() {
        BoundSymbol::Builtin(BuiltinSymbol::Name) => (),
        _ => panic!(),
    }
}

#[test]
fn parse_bound_symbol_type() {
    match syn::parse_str::<BoundSymbol>("MyType").unwrap() {
        BoundSymbol::Type(ty) => assert_eq!(ty, "MyType"),
        _ => panic!(),
    }
}

#[test]
fn parse_symbol_bound() {
    match syn::parse_str::<Symbol>("a:MyType").unwrap() {
        Symbol::Bound(name, bound_symbol) => {
            assert_eq!(name.to_string(), "a");
            match bound_symbol {
                BoundSymbol::Type(ty) => assert_eq!(ty.to_string(), "MyType"),
                _ => panic!(),
            }
        }
        _ => panic!(),
    }
}

#[test]
fn parse_symbol_literal() {
    assert!(matches!(
        syn::parse_str::<Symbol>("0xAb").unwrap(),
        Symbol::Literal(Literal::U8(0xAB))
    ));
    assert!(matches!(
        syn::parse_str::<Symbol>("0x12").unwrap(),
        Symbol::Literal(Literal::U8(0x12))
    ));
    assert!(matches!(
        syn::parse_str::<Symbol>("123:u32").unwrap(),
        Symbol::Literal(Literal::U32(123))
    ));
}

#[test]
fn parse_alternative() {
    let Production { symbols, rhs: _ } =
        syn::parse_str::<Production>("mod_:name import_name:name desc:ImportDesc = Import,")
            .unwrap();

    assert_eq!(symbols.len(), 3);
}
