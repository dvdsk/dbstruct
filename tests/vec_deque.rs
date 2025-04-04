#[dbstruct::dbstruct(db=btreemap)]
pub struct Test {
    numbers: VecDeque<u32>,
    letters: VecDeque<String>,
}

#[test]
fn clear() {
    let db = Test::new().unwrap();

    let primes = [2, 3, 5, 7];
    db.numbers().extend(&primes).unwrap();
    db.numbers().clear().unwrap();
    db.numbers().push_front(&3).unwrap();
    db.numbers().push_back(&5).unwrap();
    db.numbers().push_front(&2).unwrap();
    db.numbers().push_back(&7).unwrap();

    assert_eq!(Some(7), db.numbers().pop_back().unwrap());
    assert_eq!(Some(2), db.numbers().pop_front().unwrap());
}

mod given_empty {
    use super::*;

    #[test]
    fn len_is_zero() {
        let db = Test::new().unwrap();
        assert_eq!(db.numbers().len(), 0);
    }

    #[test]
    fn push_increases_the_len() {
        let db = Test::new().unwrap();
        db.numbers().push_back(&42).unwrap();
        assert_eq!(db.numbers().len(), 1)
    }

    #[test]
    fn pop_return_none() {
        let db = Test::new().unwrap();
        let elem = db.numbers().pop_back().unwrap();
        assert_eq!(elem, None)
    }
}

mod push_pop_back {
    use super::*;

    #[test]
    fn len_is_two() {
        let db = Test::new().unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().push_back(&43).unwrap();

        assert_eq!(db.numbers().len(), 2);
    }

    #[test]
    fn element_pop_in_the_right_order() {
        let db = Test::new().unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().push_back(&43).unwrap();

        assert_eq!(db.numbers().pop_back().unwrap(), Some(43));
        assert_eq!(db.numbers().pop_back().unwrap(), Some(42));
    }

    #[test]
    fn third_pop_is_none() {
        let db = Test::new().unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().push_back(&43).unwrap();

        db.numbers().pop_back().unwrap();
        db.numbers().pop_back().unwrap();
        let elem = db.numbers().pop_back().unwrap();
        assert_eq!(elem, None)
    }
}

mod push_pop_front {
    use super::*;

    #[test]
    fn len_is_two() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&42).unwrap();
        db.numbers().push_front(&43).unwrap();

        assert_eq!(db.numbers().len(), 2);
    }

    #[test]
    fn element_pop_in_the_right_order() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&42).unwrap();
        db.numbers().push_front(&43).unwrap();

        assert_eq!(db.numbers().pop_front().unwrap(), Some(43));
        assert_eq!(db.numbers().pop_front().unwrap(), Some(42));
    }

    #[test]
    fn third_pop_is_none() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&42).unwrap();
        db.numbers().push_front(&43).unwrap();

        db.numbers().pop_front().unwrap();
        db.numbers().pop_front().unwrap();
        let elem = db.numbers().pop_front().unwrap();
        assert_eq!(elem, None)
    }
}

mod combined_front_back {
    use super::*;

    #[test]
    fn len_is_correct() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&42).unwrap();
        assert_eq!(db.numbers().len(), 1);

        db.numbers().push_back(&43).unwrap();
        assert_eq!(db.numbers().len(), 2);

        db.numbers().push_front(&41).unwrap();
        assert_eq!(db.numbers().len(), 3);
    }

    #[test]
    fn element_pop_in_the_right_order() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&42).unwrap();
        db.numbers().push_back(&43).unwrap();
        db.numbers().push_front(&41).unwrap();

        assert_eq!(db.numbers().pop_front().unwrap(), Some(41));
        assert_eq!(db.numbers().pop_back().unwrap(), Some(43));
        assert_eq!(db.numbers().pop_front().unwrap(), Some(42));
    }

    #[test]
    fn fourth_pop_is_none() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&42).unwrap();
        db.numbers().push_back(&43).unwrap();
        db.numbers().push_front(&41).unwrap();

        db.numbers().pop_front().unwrap();
        db.numbers().pop_front().unwrap();
        db.numbers().pop_front().unwrap();
        let elem = db.numbers().pop_front().unwrap();
        assert_eq!(elem, None)
    }

    #[test]
    fn access_front_via_back_pop() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&43).unwrap();
        db.numbers().push_front(&42).unwrap();
        db.numbers().push_front(&41).unwrap();

        assert_eq!(db.numbers().pop_back().unwrap(), Some(43));
        assert_eq!(db.numbers().pop_back().unwrap(), Some(42));
        assert_eq!(db.numbers().pop_back().unwrap(), Some(41));
    }

    #[test]
    fn clear_then_push() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&43).unwrap();
        db.numbers().push_front(&42).unwrap();
        db.numbers().clear().unwrap();
        assert!(db.numbers().is_empty());
        db.numbers().push_front(&42).unwrap();
        db.numbers().push_back(&43).unwrap();
        assert_eq!(
            db.numbers().iter().collect::<Result<Vec<_>, _>>().unwrap(),
            vec![42, 43]
        );

        let db = Test::new().unwrap();
        db.numbers().push_back(&43).unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().clear().unwrap();
        assert!(db.numbers().is_empty());
        db.numbers().push_front(&42).unwrap();
        db.numbers().push_back(&43).unwrap();
        assert_eq!(
            db.numbers().iter().collect::<Result<Vec<_>, _>>().unwrap(),
            vec![42, 43]
        );
    }

    #[test]
    fn pop_clears_from_both_sides() {
        let db = Test::new().unwrap();
        db.numbers().push_front(&43).unwrap();
        db.numbers().push_front(&42).unwrap();
        db.numbers().pop_back().unwrap();
        db.numbers().pop_back().unwrap();
        assert!(db.numbers().is_empty());

        db.numbers().push_front(&42).unwrap();
        db.numbers().push_back(&43).unwrap();
        assert_eq!(
            db.numbers().iter().collect::<Result<Vec<_>, _>>().unwrap(),
            vec![42, 43]
        );

        let db = Test::new().unwrap();
        db.numbers().push_back(&43).unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().pop_front().unwrap();
        db.numbers().pop_front().unwrap();
        assert!(db.numbers().is_empty());

        db.numbers().push_back(&43).unwrap();
        db.numbers().push_front(&42).unwrap();
        assert_eq!(
            db.numbers().iter().collect::<Result<Vec<_>, _>>().unwrap(),
            vec![42, 43]
        );
    }
}

mod iterator {
    use super::*;

    #[test]
    fn trivial() {
        let db = Test::new().unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().push_back(&13).unwrap();
        db.numbers().push_back(&7).unwrap();

        let mut sum = 0;
        for elem in &db.numbers() {
            sum += elem.unwrap();
        }
        assert_eq!(sum, 62);
    }

    #[test]
    fn push_back_post_iter() {
        let db = Test::new().unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().push_back(&13).unwrap();

        let list = db.numbers();
        let iter = list.into_iter();
        db.numbers().push_back(&7).unwrap();

        let mut sum = 0;
        for elem in iter {
            sum += elem.unwrap();
        }
        assert_eq!(sum, 62);
    }

    #[test]
    fn pop_back_post_iter_is_seen() {
        let db = Test::new().unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().push_back(&13).unwrap();

        let mut sum = 0;
        let list = db.numbers();
        let iter = list.into_iter();
        db.numbers().pop_back().unwrap();

        for elem in iter {
            sum += elem.unwrap();
        }
        assert_eq!(sum, 42);
    }

    #[test]
    fn pop_back_during_iter() {
        let db = Test::new().unwrap();
        db.numbers().push_back(&42).unwrap();
        db.numbers().push_back(&13).unwrap();

        let list = db.numbers();
        let mut iter = list.into_iter();
        iter.next();
        iter.next();
        db.numbers().pop_back().unwrap();
        assert!(iter.next().is_none());
    }
}

mod extend {
    use super::*;

    #[test]
    fn push_str_slices() {
        let db = Test::new().unwrap();

        let iter = ["a", "b", "c", "d"];
        db.letters().extend(iter).unwrap();
        assert_eq!(db.letters().len(), 4);
        assert_eq!(db.letters().pop_back().unwrap(), Some("d".to_string()));
    }

    #[test]
    fn push_strings() {
        let db = Test::new().unwrap();

        let iter = [
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        ];
        db.letters().extend(&iter).unwrap();
    }
}
