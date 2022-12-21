use std::path::Path;
use ts_rs::TS;

pub fn export_type<T: TS>() {
    let file_name = format!("{}.d.ts", T::name());
    let path = Path::new("../ts-types").join(file_name);
    T::export_to(path).unwrap()
}
}
