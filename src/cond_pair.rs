pub struct CondPair<T> {
    t: T,
    f: T,
    cond: bool,
}

impl<T> CondPair<T> {
    pub fn new(a: T, b: T) -> Self {
        Self {
            t: a,
            f: b,
            cond: true,
        }
    }

    pub fn get(&self) -> (&T, &T) {
        (&self.t, &self.f)
    }

    pub fn get_true(&self) -> &T {
        if self.cond {
            &self.t
        } else {
            &self.f
        }
    }

    pub fn get_false(&self) -> &T {
        if self.cond {
            &self.f
        } else {
            &self.t
        }
    }

    pub fn get_mut_true(&mut self) -> &mut T {
        if self.cond {
            &mut self.t
        } else {
            &mut self.f
        }
    }

    pub fn get_mut_false(&mut self) -> &mut T {
        if self.cond {
            &mut self.f
        } else {
            &mut self.t
        }
    }

    pub fn update_cond<F>(&mut self, f: F)
    where
        F: FnOnce(&T, &T) -> bool,
    {
        self.cond = f(&self.t, &self.f);
    }
}
