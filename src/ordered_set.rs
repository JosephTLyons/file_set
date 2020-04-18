use std::clone::Clone;
use std::cmp::{Ord, PartialEq};
use std::convert::TryFrom;

#[derive(Default)]
pub struct OrderedSet<T> {
    items: Vec<T>,
}

impl<T: Ord + PartialEq + Clone> OrderedSet<T> {
    pub fn new() -> OrderedSet<T> {
        OrderedSet { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) -> Result<(), &'static str> {
        if self.items.contains(&item) {
            return Err("Cannot add an item to set that already exists in the set");
        }

        self.items.push(item);

        Ok(())
    }

    pub fn intersection(&self, other: &OrderedSet<T>) -> OrderedSet<T> {
        self.intersection_difference_base(other, true)
    }

    pub fn difference(&self, other: &OrderedSet<T>) -> OrderedSet<T> {
        self.intersection_difference_base(other, false)
    }

    fn intersection_difference_base(
        &self,
        other: &OrderedSet<T>,
        should_compute_intersection: bool,
    ) -> OrderedSet<T> {
        OrderedSet {
            items: self
                .items
                .clone()
                .into_iter()
                .filter(|x| other.items.contains(x) == should_compute_intersection)
                .collect(),
        }
    }

    pub fn reverse(&mut self) -> OrderedSet<T> {
        self.items.reverse();
        OrderedSet {
            items: self.items.clone(),
        }
    }

    pub fn is_disjoint(&self, other: &OrderedSet<T>) -> bool {
        self.intersection(&other).to_vec().is_empty()
    }

    pub fn to_vec(&self) -> Vec<T> {
        self.items.clone()
    }
}

impl<T: Clone> Clone for OrderedSet<T> {
    fn clone(&self) -> OrderedSet<T> {
        OrderedSet {
            items: self.items.clone(),
        }
    }
}

impl<T: PartialEq> TryFrom<Vec<T>> for OrderedSet<T> {
    type Error = &'static str;

    fn try_from(vec: Vec<T>) -> Result<OrderedSet<T>, Self::Error> {
        for item in &vec {
            if vec.iter().filter(|&n| n == item).count() > 1 {
                return Err("All elements of the set must be unique");
            }
        }

        Ok(OrderedSet { items: vec })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_vec_no_duplicates_test() {
        let ordered_set = OrderedSet::try_from(["a", "b", "c"].to_vec()).unwrap();
        assert_eq!(ordered_set.to_vec(), ["a", "b", "c"].to_vec());
    }

    #[test]
    fn try_from_vec_duplicates_test() {
        let ordered_set = OrderedSet::try_from(["a", "b", "c", "a"].to_vec());
        assert!(ordered_set.is_err());
    }

    #[test]
    fn push_no_duplicates_test() {
        let mut ordered_set: OrderedSet<u8> = OrderedSet::new();

        ordered_set.push(1).unwrap();
        ordered_set.push(2).unwrap();

        assert_eq!(ordered_set.to_vec(), [1, 2].to_vec());
    }

    #[test]
    fn push_duplicates_test() {
        let mut ordered_set: OrderedSet<String> = OrderedSet::new();

        ordered_set.push(String::from("Dog")).unwrap();
        ordered_set.push(String::from("Cat")).unwrap();

        assert!(ordered_set.push(String::from("Dog")).is_err());
    }

    #[test]
    fn intersection_test() {
        let mut ordered_set_1: OrderedSet<u8> = OrderedSet::new();

        ordered_set_1.push(1).unwrap();
        ordered_set_1.push(2).unwrap();
        ordered_set_1.push(9).unwrap();
        ordered_set_1.push(3).unwrap();

        let mut ordered_set_2: OrderedSet<u8> = OrderedSet::new();

        ordered_set_2.push(10).unwrap();
        ordered_set_2.push(2).unwrap();
        ordered_set_2.push(9).unwrap();
        ordered_set_2.push(11).unwrap();

        let intersection_vec = ordered_set_1.intersection(&ordered_set_2).to_vec();

        assert!(intersection_vec.len() == 2);
        assert!(intersection_vec.contains(&2));
        assert!(intersection_vec.contains(&9));
    }

    #[test]
    fn difference_test() {
        let mut ordered_set_1: OrderedSet<u8> = OrderedSet::new();

        ordered_set_1.push(1).unwrap();
        ordered_set_1.push(2).unwrap();
        ordered_set_1.push(9).unwrap();
        ordered_set_1.push(3).unwrap();

        let mut ordered_set_2: OrderedSet<u8> = OrderedSet::new();

        ordered_set_2.push(10).unwrap();
        ordered_set_2.push(2).unwrap();
        ordered_set_2.push(9).unwrap();
        ordered_set_2.push(11).unwrap();

        let diference_vec = ordered_set_1.difference(&ordered_set_2).to_vec();

        assert!(diference_vec.len() == 2);
        assert!(diference_vec.contains(&1));
        assert!(diference_vec.contains(&3));
    }

    #[test]
    fn disjoin_test() {
        let mut ordered_set_1: OrderedSet<u8> = OrderedSet::new();

        ordered_set_1.push(1).unwrap();
        ordered_set_1.push(2).unwrap();
        ordered_set_1.push(9).unwrap();
        ordered_set_1.push(3).unwrap();

        let mut ordered_set_2: OrderedSet<u8> = OrderedSet::new();

        ordered_set_2.push(10).unwrap();
        ordered_set_2.push(2).unwrap();
        ordered_set_2.push(9).unwrap();
        ordered_set_2.push(11).unwrap();

        assert_eq!(ordered_set_1.is_disjoint(&ordered_set_2), false);
    }

    #[test]
    fn reverse_test() {
        let mut ordered_set: OrderedSet<u8> = OrderedSet::new();

        ordered_set.push(1).unwrap();
        ordered_set.push(2).unwrap();
        ordered_set.push(9).unwrap();
        ordered_set.push(3).unwrap();

        ordered_set.reverse();

        assert_eq!(ordered_set.to_vec(), [3, 9, 2, 1].to_vec());
    }
}
