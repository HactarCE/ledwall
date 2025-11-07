use std::ops::{Deref, DerefMut};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(align(8))]
pub struct ArrayVec<T, const CAP: usize> {
    contents: [T; CAP],
    len: u8,
}

impl<T: Default, const CAP: usize> Default for ArrayVec<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default, const CAP: usize> ArrayVec<T, CAP> {
    pub fn new() -> Self {
        Self {
            len: 0,
            contents: std::array::from_fn(|_| T::default()),
        }
    }
}

impl<T, const CAP: usize> Deref for ArrayVec<T, CAP> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.contents[..self.len as usize]
    }
}

impl<T, const CAP: usize> DerefMut for ArrayVec<T, CAP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.contents[..self.len as usize]
    }
}

impl<T, const CAP: usize> ArrayVec<T, CAP> {
    pub fn push(&mut self, element: T) {
        self.contents[self.len as usize] = element;
        self.len += 1;
    }

    pub fn try_push(&mut self, element: T) -> Result<(), ()> {
        if (self.len as usize) < CAP {
            self.push(element);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn remove(&mut self, index: usize) {
        assert!(index < self.len as usize);
        self[index..].rotate_left(1);
        self.len -= 1;
    }
}

impl<T: Default, const CAP: usize> FromIterator<T> for ArrayVec<T, CAP> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut ret = Self::default();
        for element in iter {
            ret.push(element);
        }
        ret
    }
}
