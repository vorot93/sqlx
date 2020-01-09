use std::fmt::Display;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use syn::{Ident, Path};

use quote::quote;
use tokio_sqlx::{types::HasTypeMetadata, Connection};

use super::{args, output, QueryMacroInput};
use crate::database::DatabaseExt;

/// Given an input like `query!("SELECT * FROM accounts WHERE account_id > ?", account_id)`,
/// expand to an anonymous record
pub async fn expand_query<C: Connection>(
    input: QueryMacroInput,
    mut conn: C,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as HasTypeMetadata>::TypeId: Display,
{
    let describe = input.describe_validate(&mut conn).await?;
    let sql = &input.source;

    let args = args::quote_args(&input, &describe)?;

    if describe.result_columns.is_empty() {
        return Ok(quote! {{
            #args
            tokio_sqlx::query_as_mapped(#sql, |_| Ok(())).bind_all(args)
        }});
    }

    let columns = output::columns_to_rust(&describe)?;

    // record_type will be wrapped in parens which the compiler ignores without a trailing comma
    // e.g. (Foo) == Foo but (Foo,) = one-element tuple
    // and giving an empty stream for record_type makes it unit `()`
    let record_type: Path = Ident::new("Record", Span::call_site()).into();

    let record_fields = columns
        .iter()
        .map(
            |&output::RustColumn {
                 ref ident,
                 ref type_,
             }| quote!(#ident: #type_,),
        )
        .collect::<TokenStream>();

    let output = output::quote_query_as::<C::Database>(sql, &record_type, &columns);

    Ok(quote! {{
        #[derive(Debug)]
        struct #record_type {
            #record_fields
        }

        #args

        #output.bind_all(args)
    }})
}
