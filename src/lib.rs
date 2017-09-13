#[macro_use] extern crate ruru;
#[macro_use] extern crate lazy_static;

extern crate calamine;

use ruru::{Array, Class, RString, Float, Fixnum, Boolean, Object, AnyObject, NilClass, VM};
use calamine::{Sheets, DataType};

pub struct Reader {
    file_name: String,
    rows: Array
}

impl Reader {
    fn new(file_name: String) -> Self {
        if file_name.is_empty() {
            VM::raise(Class::from_existing("RuntimeError"), "file name can't be empty")
        }

        let mut this = Reader {
            file_name: file_name,
            rows: Array::new()
        };

        let file = Sheets::open(&this.file_name);
        if let Err(ref error) = file {
            VM::raise(Class::from_existing("RuntimeError"), error.description());
        }

        let sheet = file.unwrap().worksheet_range_by_index(0).unwrap();

        for row in sheet.rows() {
            let mut new_row = Array::with_capacity(sheet.width());

            for (_, c) in row.iter().enumerate() {
                match *c {
                    DataType::Error(_)      => new_row.push(NilClass::new()),
                    DataType::String(ref s) => new_row.push(RString::new(s)),
                    DataType::Empty         => new_row.push(NilClass::new()),
                    DataType::Float(ref f)  => new_row.push(Float::new(*f)),
                    DataType::Int(ref i)    => new_row.push(Fixnum::new(*i)),
                    DataType::Bool(ref b)   => new_row.push(Boolean::new(*b))
                };
            }

            this.rows.push(new_row);
        }
        this
    }

    fn get_rows(&self) -> AnyObject {
        self.rows.to_any_object()
    }
}

wrappable_struct!(Reader, ReaderWrapper, READER_WRAPPER);

class!(Xlsx);

methods!(
    Xlsx,
    itself,

    fn ruby_xlsx_new(file_name: RString) -> AnyObject {
        if let Err(ref error) = file_name {
            VM::raise(Class::from_existing("RuntimeError"), "file name should be a string");
        }

        let xlsx_reader = Reader::new(file_name.unwrap().to_string());
        Class::from_existing("Xlsx").wrap_data(xlsx_reader, &*READER_WRAPPER)
    }

    fn ruby_xlsx_rows() -> AnyObject {
        itself.get_data(&*READER_WRAPPER).get_rows()
    }
);

#[no_mangle]
#[allow(non_snake_case)]
pub extern fn Init_libfastsheet() {
    Class::new("Xlsx", None).define(|itself| {
        itself.def_self("new", ruby_xlsx_new);
        itself.def("rows", ruby_xlsx_rows)
    });
}
