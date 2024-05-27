use chrono::{DateTime, Utc};
use sqlx::{Database, Encode, QueryBuilder, Type};

#[trait_bounds::each(String, i16, i32, f32, f64, DateTime<Utc>: Type<DB> + for<'a> Encode<'a, DB>)]
fn easy_generic_over_db<DB>(query_builder: &mut QueryBuilder<'_, DB>)
where
    DB: Database,
{
    todo!()
}
