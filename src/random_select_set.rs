use std::iter;

use rand::RngExt;

pub struct RandomSelectSet {
    len: usize,
    tree: Box<[usize]>,
}

impl RandomSelectSet {
    pub fn new_full(len: usize) -> Self {
        let len_orig = len;
        let len = len.next_power_of_two();
        let mut tree = iter::repeat_n(0, len).chain(iter::repeat_n(1, len_orig)).chain(iter::repeat_n(0, len - len_orig)).collect::<Box<[_]>>();
        assert_eq!(len * 2, tree.len());
        for i in (1..len).rev() {
            tree[i] = tree[i * 2] + tree[i * 2 + 1];
        }
        Self {
            len,
            tree
        }
    }

    pub fn new_empty(len: usize) -> Self {
        let len = len.next_power_of_two();
        Self {
            len,
            tree: iter::repeat_n(0, len * 2).collect(),
        }
    }

    pub fn contains(&self, i: usize) -> bool {
        self.tree[self.len + i] != 0
    }

    pub fn is_empty(&self) -> bool {
        self.tree[1] == 0
    }

    pub fn len(&self) -> usize {
        self.tree[1]
    }

    fn random_index<R: ?Sized + RngExt>(&self, rng: &mut R) -> usize {
        assert!(!self.is_empty());
        let mut cur = 1;
        while cur < self.len {
            let a = self.tree[cur * 2];
            let b = self.tree[cur * 2 + 1];
            cur *= 2;
            if a == 0 {
                assert!(b != 0);
                cur += 1;
                continue;
            }
            if b == 0 {
                assert!(a != 0);
                continue;
            }
            let rn = rng.random_range(0..a+b);
            if rn < b {
                cur += 1;
            }
        }
        assert!((self.len..self.len * 2).contains(&cur));
        assert!(self.tree[cur] > 0);
        cur
    }

    pub fn take_random<R: ?Sized + RngExt>(&mut self, rng: &mut R) -> usize {
        let idx = self.random_index(rng);
        self.tree[idx] -= 1;
        idx - self.len
    }

    pub fn insert(&mut self, i: usize) -> bool {
        let idx = i + self.len;
        if self.tree[idx] == 0 {
            self.tree[idx] = 1;
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, i: usize) -> bool {
        let idx = i + self.len;
        if self.tree[idx] > 0 {
            self.tree[idx] -= 1;
            true
        } else {
            false
        }
    }
}