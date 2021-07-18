#[cfg(test)]
mod device_tests {
    mod linux {
        use rustic_stack::device::linux;
        // use std::fs::File;
        // use std::io;
        // use std::io::prelude::*;
        // use std::path::Path;

        #[test]
        fn open_tap() {
            let r = linux::open_tap();
            assert_eq!((), r.unwrap());
        }

        // #[test]
        // fn read_tap() {
        //     let mut buf = [0; linux::RW_BUF_SIZE];
        //     linux::read_tap(&mut buf, linux::RW_BUF_SIZE);
        // }

        // #[test]
        // fn write_tap() {
        //     let mut buf = [b'a'; linux::RW_BUF_SIZE];
        //     linux::write_tap(&mut buf, linux::RW_BUF_SIZE);
        // }
    }
}
