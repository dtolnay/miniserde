use de::{Map, Seq, Visitor};
use error::Result;

impl Visitor {
    pub fn ignore() -> &'static mut Visitor {
        careful!(&mut Ignore as &mut Ignore)
    }
}

struct Ignore;

impl Visitor for Ignore {
    fn null(&mut self) -> Result<()> {
        Ok(())
    }

    fn boolean(&mut self, _b: bool) -> Result<()> {
        Ok(())
    }

    fn string(&mut self, _s: &str) -> Result<()> {
        Ok(())
    }

    fn negative(&mut self, _n: i64) -> Result<()> {
        Ok(())
    }

    fn nonnegative(&mut self, _n: u64) -> Result<()> {
        Ok(())
    }

    fn float(&mut self, _n: f64) -> Result<()> {
        Ok(())
    }

    fn seq(&mut self) -> Result<Box<Seq + '_>> {
        Ok(Box::new(Ignore))
    }

    fn map(&mut self) -> Result<Box<Map + '_>> {
        Ok(Box::new(Ignore))
    }
}

impl Seq for Ignore {
    fn element(&mut self) -> Result<&mut Visitor> {
        Ok(Visitor::ignore())
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Map for Ignore {
    fn key(&mut self, _k: &str) -> Result<&mut Visitor> {
        Ok(Visitor::ignore())
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}
