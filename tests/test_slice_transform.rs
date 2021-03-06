extern crate ckb_rocksdb as rocksdb;

use crate::rocksdb::{prelude::*, SliceTransform, TemporaryDBPath};

#[test]
pub fn test_slice_transform() {
    let n = TemporaryDBPath::new();
    {
        let a1: Box<[u8]> = key(b"aaa1");
        let a2: Box<[u8]> = key(b"aaa2");
        let b1: Box<[u8]> = key(b"bbb1");
        let b2: Box<[u8]> = key(b"bbb2");

        fn first_three<'a>(k: &'a [u8]) -> &'a [u8] {
            &k[..3]
        }

        let prefix_extractor = SliceTransform::create("first_three", first_three, None);

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_prefix_extractor(prefix_extractor);

        let db = DB::open(&opts, &n).unwrap();

        assert!(db.put(&*a1, &*a1).is_ok());
        assert!(db.put(&*a2, &*a2).is_ok());
        assert!(db.put(&*b1, &*b1).is_ok());
        assert!(db.put(&*b2, &*b2).is_ok());

        fn cba(input: &[u8]) -> Box<[u8]> {
            input.to_vec().into_boxed_slice()
        }

        fn key(k: &[u8]) -> Box<[u8]> {
            k.to_vec().into_boxed_slice()
        }

        {
            let expected = vec![(cba(&a1), cba(&a1)), (cba(&a2), cba(&a2))];
            let a_iterator = db.prefix_iterator(b"aaa");
            assert_eq!(a_iterator.collect::<Vec<_>>(), expected)
        }

        {
            let expected = vec![(cba(&b1), cba(&b1)), (cba(&b2), cba(&b2))];
            let b_iterator = db.prefix_iterator(b"bbb");
            assert_eq!(b_iterator.collect::<Vec<_>>(), expected)
        }
    }
}

#[test]
fn test_no_in_domain() {
    fn extract_suffix(slice: &[u8]) -> &[u8] {
        if slice.len() > 4 {
            &slice[slice.len() - 4..slice.len()]
        } else {
            slice
        }
    }

    let db_path = TemporaryDBPath::new();
    {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_prefix_extractor(SliceTransform::create(
            "test slice transform",
            extract_suffix,
            None,
        ));
        opts.set_memtable_prefix_bloom_ratio(0.1);

        let db = DB::open(&opts, &db_path).unwrap();
        db.put(b"key_sfx1", b"a").unwrap();
        db.put(b"key_sfx2", b"b").unwrap();

        assert_eq!(
            db.get(b"key_sfx1").unwrap().unwrap().as_ref(),
            b"a".as_ref()
        );
    }
}
