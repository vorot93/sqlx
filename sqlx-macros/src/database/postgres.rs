impl_database_ext! {
    tokio_sqlx::Postgres {
        bool,
        String,
        i16,
        i32,
        i64,
        f32,
        f64,

        #[cfg(feature = "uuid")]
        tokio_sqlx::types::Uuid,

        #[cfg(feature = "chrono")]
        tokio_sqlx::types::chrono::NaiveTime,

        #[cfg(feature = "chrono")]
        tokio_sqlx::types::chrono::NaiveDate,

        #[cfg(feature = "chrono")]
        tokio_sqlx::types::chrono::NaiveDateTime,

        #[cfg(feature = "chrono")]
        tokio_sqlx::types::chrono::DateTime<tokio_sqlx::types::chrono::Utc> | tokio_sqlx::types::chrono::DateTime<_>,
    },
    ParamChecking::Strong
}
