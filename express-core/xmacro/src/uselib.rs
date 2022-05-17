use syn::{self, parse::Parse, spanned::Spanned, Token};

pub struct Library {
    pub ctx: syn::ExprPath,
    pub root: syn::ExprPath,
    pub constants: Paths,
    pub functions: Paths,
}

pub struct Paths {
    pub values: Vec<(String, syn::ExprPath, syn::Ident)>,
}

fn eat_token(input: syn::parse::ParseStream, name: &str) -> syn::Result<()> {
    let ctx_tok = input.parse::<syn::Ident>()?;
    if ctx_tok != name {
        return Err(syn::Error::new(
            ctx_tok.span(),
            format!("Expected token {} but found {}", name, ctx_tok),
        ));
    }
    Ok(())
}

fn parse_path_block(input: syn::parse::ParseStream, name: &str) -> syn::Result<Paths> {
    let tok: Option<syn::Ident> = input.parse()?;
    if let Some(tok) = tok {
        if tok != name {
            return Err(syn::Error::new(
                tok.span(),
                format!("Expected token '{}' but recieved: {}", name, tok),
            ));
        }
    } else {
        return Ok(Paths { values: vec![] });
    }
    let block = input.parse::<syn::Block>()?;
    let mut values = Vec::with_capacity(block.stmts.len());
    for stmt in block.stmts {
        if let syn::Stmt::Semi(syn::Expr::Path(mut value), _) = stmt {
            let (last, _) = value.path.segments.pop().unwrap().into_tuple();
            let name = last.ident.to_string();
            values.push((name, value, last.ident));
        } else {
            return Err(syn::Error::new(
                stmt.span(),
                format!("Recieved unrecognized path. Try adding semicolon ';'"),
            ));
        }
    }
    Ok(Paths { values })
}

impl Parse for Library {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        eat_token(input, "context")?;
        let ctx = input.parse::<syn::ExprPath>()?;
        input.parse::<Token![;]>()?;
        eat_token(input, "library")?;
        let root = input.parse::<syn::ExprPath>()?;
        input.parse::<Token![;]>()?;
        let constants: Paths = parse_path_block(input, "constants")?;
        let functions: Paths = parse_path_block(input, "functions")?;

        Ok(Library {
            ctx,
            root,
            constants,
            functions,
        })
    }
}
