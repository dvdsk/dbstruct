#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    list: VecDeque<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir::TempDir::new("vecdeque_example")?;
    let db = Test::open_path(&dir)?;

    db.list().push_front(&2)?;
    db.list().push_front(&1)?;
    db.list().push_back(&3)?;

    // Dropping the db here simulates the program
    // stopping and restarting
    std::mem::drop(db);
    let db = Test::open_path(&dir)?;

    let res: Vec<_> = db.list().into_iter().collect::<Result<_, _>>()?;
    assert_eq!(res, vec![1, 2, 3]);

    Ok(())
}
