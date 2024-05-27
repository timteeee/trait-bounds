#[trait_bounds::each(String, for<'a> &'a str: From<T>)]
fn my_func<T>() -> T {
    todo!()
}
