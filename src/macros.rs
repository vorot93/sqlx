/// Statically checked SQL query with `println!()` style syntax.
///
/// This expands to an instance of [QueryAs] that outputs an ad-hoc anonymous struct type,
/// if the query has output columns, or `()` (unit) otherwise:
///
/// ```rust
/// # #[cfg(feature = "mysql")]
/// # #[tokio::main]
/// # async fn main() -> tokio_sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = tokio_sqlx::mysql::connect(db_url).await?;
/// // let mut conn = <impl tokio_sqlx::Executor>;
/// let account = tokio_sqlx::query!("select (1) as id, 'Herp Derpinson' as name")
///     .fetch_one(&mut conn)
///     .await?;
///
/// // anonymous struct has `#[derive(Debug)]` for convenience
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(not(feature = "mysql"))]
/// # fn main() {}
/// ```
///
/// ## Query Arguments
/// Like `println!()` and the other formatting macros, you can add bind parameters to your SQL
/// and this macro will typecheck passed arguments and error on missing ones:
///
/// ```rust
/// # #[cfg(feature = "mysql")]
/// # #[tokio::main]
/// # async fn main() -> tokio_sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = tokio_sqlx::mysql::connect(db_url).await?;
/// // let mut conn = <impl tokio_sqlx::Executor>;
/// let account = tokio_sqlx::query!(
///         // just pretend "accounts" is a real table
///         "select * from (select (1) as id, 'Herp Derpinson' as name) accounts where id = ?",
///         1i32
///     )
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
/// # Ok(())
/// # }
/// #
/// # #[cfg(not(feature = "mysql"))]
/// # fn main() {}
/// ```
///
/// Bind parameters in the SQL string are specific to the database backend:
///
/// * Postgres: `$N` where `N` is the 1-based positional argument index
/// * MySQL: `?` which matches arguments in order that it appears in the query
///
/// ## Requirements
/// * The `DATABASE_URL` environment variable must be set at build-time to point to a database
/// server with the schema that the query string will be checked against. (All variants of
/// `query!()` use [dotenv] so this can be in a `.env` file instead.)
///
/// * The query must be a string literal or else it cannot be introspected (and thus cannot
/// be dynamic or the result of another macro).
///
/// * The `QueryAs` instance will be bound to the same database type as `query!()` was compiled
/// against (e.g. you cannot build against a Postgres database and then run the query against
/// a MySQL database).
///
///     * The schema of the database URL (e.g. `postgres://` or `mysql://`) will be used to
///       determine the database type.
///
/// [dotenv]: https://crates.io/crates/dotenv
/// ## See Also
/// * [query_as!] if you want to use a struct you can name,
/// * [query_file!] if you want to define the SQL query out-of-line,
/// * [query_file_as!] if you want both of the above.
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! query (
    // the emitted item for `#[proc_macro_hack]` doesn't look great in docs
    // plus this might let IDEs hint at the syntax
    // `#[allow(dead_code)]` to silence the `enum ProcMacroHack` error
    ($query:literal) => (#[allow(dead_code)] {
        $crate::query_!($query)
    });
    ($query:literal, $($args:tt)*) => (#[allow(dead_code)]{
        #![allow(dead_code)]
        $crate::query_!($query, $($args)*)
    })
);

/// A variant of [query!] where the SQL query is stored in a separate file.
///
/// Useful for large queries and potentially cleaner than multiline strings.
///
/// The syntax and requirements (see [query!]) are the same except the SQL string is replaced by a
/// file path.
///
/// The file must be relative to the project root (the directory containing `Cargo.toml`),
/// unlike `include_str!()` which uses compiler internals to get the path of the file where it
/// was invoked.
///
/// -----
///
/// `examples/queries/account-by-id.sql`:
/// ```sql
/// select * from (select (1) as id, 'Herp Derpinson' as name) accounts
/// where id = ?
/// ```
///
/// `src/my_query.rs`:
/// ```rust
/// # #[cfg(feature = "mysql")]
/// # #[tokio::main]
/// # async fn main() -> tokio_sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = tokio_sqlx::mysql::connect(db_url).await?;
/// // let mut conn = <impl tokio_sqlx::Executor>;
/// let account = tokio_sqlx::query_file!("examples/queries/account-by-id.sql", 1i32)
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(not(feature = "mysql"))]
/// # fn main() {}
/// ```
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! query_file (
    ($query:literal) => (#[allow(dead_code)]{
        $crate::query_file_!($query)
    });
    ($query:literal, $($args:tt)*) => (#[allow(dead_code)]{
        $crate::query_file_!($query, $($args)*)
    })
);

/// A variant of [query!] which takes a path to an explicitly defined struct as the output type.
///
/// This lets you return the struct from a function or add your own trait implementations.
///
/// No trait implementations are required; the macro maps rows using a struct literal
/// where the names of columns in the query are expected to be the same as the fields of the struct
/// (but the order does not need to be the same). The types of the columns are based on the
/// query and not the corresponding fields of the struct, so this is type-safe as well.
///
/// This enforces a few things:
/// * The query must output at least one column.
/// * The column names of the query must match the field names of the struct.
/// * Neither the query nor the struct may have unused fields.
///
/// The only modification to the syntax is that the struct name is given before the SQL string:
/// ```rust
/// # #[cfg(feature = "mysql")]
/// # #[tokio::main]
/// # async fn main() -> tokio_sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = tokio_sqlx::mysql::connect(db_url).await?;
/// #[derive(Debug)]
/// struct Account {
///     id: i32,
///     name: String
/// }
///
/// // let mut conn = <impl tokio_sqlx::Executor>;
/// let account = tokio_sqlx::query_as!(
///         Account,
///         "select * from (select (1) as id, 'Herp Derpinson' as name) accounts where id = ?",
///         1i32
///     )
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(not(feature = "mysql"))]
/// # fn main() {}
/// ```
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! query_as (
    ($out_struct:path, $query:literal) => (#[allow(dead_code)] {
        $crate::query_as_!($out_struct, $query)
    });
    ($out_struct:path, $query:literal, $($args:tt)*) => (#[allow(dead_code)] {
        $crate::query_as_!($out_struct, $query, $($args)*)
    })
);

/// Combines the syntaxes of [query_as!] and [query_file!].
///
/// Enforces requirements of both macros; see them for details.
///
/// ```rust
/// # #[cfg(feature = "mysql")]
/// # #[tokio::main]
/// # async fn main() -> tokio_sqlx::Result<()>{
/// # let db_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = tokio_sqlx::mysql::connect(db_url).await?;
/// #[derive(Debug)]
/// struct Account {
///     id: i32,
///     name: String
/// }
///
/// // let mut conn = <impl tokio_sqlx::Executor>;
/// let account = tokio_sqlx::query_file_as!(Account, "examples/queries/account-by-id.sql", 1i32)
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{:?}", account);
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(not(feature = "mysql"))]
/// # fn main() {}
/// ```
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! query_file_as (
    ($out_struct:path, $query:literal) => (#[allow(dead_code)] {
        $crate::query_file_as_!($out_struct, $query)
    });
    ($out_struct:path, $query:literal, $($args:tt)*) => (#[allow(dead_code)] {
        $crate::query_file_as_!($out_struct, $query, $($args)*)
    })
);
