impl_database_ext! {
    tokio_sqlx::MySql {
        bool,
        String,
        i16,
        i32,
        i64,
        f32,
        f64
    },
    ParamChecking::Weak
}
