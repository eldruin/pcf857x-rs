#[macro_export]
macro_rules! expect_err {
    ($result:expr, $error:ident) => {
        match $result {
            Err(Error::$error) => (),
            _ => panic!(),
        }
    };
}
