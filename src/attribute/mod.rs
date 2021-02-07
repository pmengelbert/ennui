pub trait Attribute<T: Copy + Eq> {
    fn attr(&self) -> Vec<T>;
    fn set_attr(&mut self, q: T);
    fn unset_attr(&mut self, q: T);

    fn is(&self, a: T) -> bool {
        self.attr().contains(&a)
    }

    fn is_all(&self, ats: &[T]) -> bool {
        for a in ats {
            if !self.attr().contains(a) {
                return false;
            }
        }

        true
    }
}
