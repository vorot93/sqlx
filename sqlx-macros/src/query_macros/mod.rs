use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::quote;

pub use input::{QueryAsMacroInput, QueryMacroInput};
pub use query::expand_query;

use crate::database::DatabaseExt;

use tokio_sqlx::types::HasTypeMetadata;
use tokio_sqlx::Connection;

mod args;
mod input;
mod output;
mod query;

pub async fn expand_query_file<C: Connection>(
    input: QueryMacroInput,
    conn: C,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as HasTypeMetadata>::TypeId: Display,
{
    expand_query(input.expand_file_src().await?, conn).await
}

pub async fn expand_query_as<C: Connection>(
    input: QueryAsMacroInput,
    mut conn: C,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as HasTypeMetadata>::TypeId: Display,
{
    let describe = input.query_input.describe_validate(&mut conn).await?;

    if describe.result_columns.is_empty() {
        return Err(syn::Error::new(
            input.query_input.source_span,
            "query must output at least one column",
        )
        .into());
    }

    let args_tokens = args::quote_args(&input.query_input, &describe)?;

    let columns = output::columns_to_rust(&describe)?;
    let output = output::quote_query_as::<C::Database>(
        &input.query_input.source,
        &input.as_ty.path,
        &columns,
    );

    Ok(quote! {{
        #args_tokens
        #output.bind_all(args)
    }})
}

pub async fn expand_query_file_as<C: Connection>(
    input: QueryAsMacroInput,
    conn: C,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as HasTypeMetadata>::TypeId: Display,
{
    expand_query_as(input.expand_file_src().await?, conn).await
}
