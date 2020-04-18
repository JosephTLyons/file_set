use std::clone::Clone;
use std::cmp::{Ord, PartialEq};
use std::convert::TryFrom;

#[derive(Default)]
pub struct OrderableSet<T> {
    items: Vec<T>,
}

impl<T: Ord + PartialEq + Clone> OrderableSet<T> {
    pub fn new() -> OrderableSet<T> {
        OrderableSet { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) -> Result<(), &'static str> {
        if self.items.contains(&item) {
            return Err("Cannot add an item to set that already exists in the set");
        }

        self.items.push(item);

        Ok(())
    }

    pub fn intersection(&self, other: &OrderableSet<T>) -> OrderableSet<T> {
        self.intersection_difference_base(other, true)
    }

    pub fn difference(&self, other: &OrderableSet<T>) -> OrderableSet<T> {
        self.intersection_difference_base(other, false)
    }

    pub fn reverse(&mut self) -> OrderableSet<T> {
        self.items.reverse();
        OrderableSet {
            items: self.items.clone(),
        }
    }

    fn intersection_difference_base(
        &self,
        other: &OrderableSet<T>,
        should_compute_intersection: bool,
    ) -> OrderableSet<T> {
        OrderableSet {
            items: self
                .items
                .clone()
                .into_iter()
                .filter(|x| other.items.contains(x) == should_compute_intersection)
                .collect(),
        }
    }

    pub fn is_disjoint(&self, other: &OrderableSet<T>) -> bool {
        self.intersection(&other).to_vec().is_empty()
    }

    pub fn to_vec(&self) -> Vec<T> {
        self.items.clone()
    }
}

impl<T: Clone> Clone for OrderableSet<T> {
    fn clone(&self) -> OrderableSet<T> {
        OrderableSet {
            items: self.items.clone(),
        }
    }
}

impl<T: PartialEq> TryFrom<Vec<T>> for OrderableSet<T> {
    type Error = &'static str;

    fn try_from(vec: Vec<T>) -> Result<OrderableSet<T>, Self::Error> {
        for item in &vec {
            if vec.iter().filter(|&n| n == item).count() > 1 {
                return Err("All elements of the set must be unique");
            }
        }

        Ok(OrderableSet { items: vec })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_vec_no_duplicates_test() {
        let orderable_set = OrderableSet::try_from(["a", "b", "c"].to_vec()).unwrap();
        assert_eq!(orderable_set.to_vec(), ["a", "b", "c"].to_vec());
    }

    #[test]
    fn try_from_vec_duplicates_test() {
        let orderable_set = OrderableSet::try_from(["a", "b", "c", "a"].to_vec());
        assert!(orderable_set.is_err());
    }

    #[test]
    fn push_no_duplicates_test() {
        let mut orderable_set: OrderableSet<u8> = OrderableSet::new();

        orderable_set.push(1).unwrap();
        orderable_set.push(2).unwrap();

        assert_eq!(orderable_set.to_vec(), [1, 2].to_vec());
    }

    #[test]
    fn push_duplicates_test() {
        let mut orderable_set: OrderableSet<String> = OrderableSet::new();

        orderable_set.push(String::from("Dog")).unwrap();
        orderable_set.push(String::from("Cat")).unwrap();

        assert!(orderable_set.push(String::from("Dog")).is_err());
    }

    #[test]
    fn intersection_test() {
        let mut orderable_set_1: OrderableSet<u8> = OrderableSet::new();

        orderable_set_1.push(1).unwrap();
        orderable_set_1.push(2).unwrap();
        orderable_set_1.push(9).unwrap();
        orderable_set_1.push(3).unwrap();

        let mut orderable_set_2: OrderableSet<u8> = OrderableSet::new();

        orderable_set_2.push(10).unwrap();
        orderable_set_2.push(2).unwrap();
        orderable_set_2.push(9).unwrap();
        orderable_set_2.push(11).unwrap();

        let intersection_vec = orderable_set_1.intersection(&orderable_set_2).to_vec();

        assert!(intersection_vec.len() == 2);
        assert!(intersection_vec.contains(&2));
        assert!(intersection_vec.contains(&9));
    }

    #[test]
    fn difference_test() {
        let mut orderable_set_1: OrderableSet<u8> = OrderableSet::new();

        orderable_set_1.push(1).unwrap();
        orderable_set_1.push(2).unwrap();
        orderable_set_1.push(9).unwrap();
        orderable_set_1.push(3).unwrap();

        let mut orderable_set_2: OrderableSet<u8> = OrderableSet::new();

        orderable_set_2.push(10).unwrap();
        orderable_set_2.push(2).unwrap();
        orderable_set_2.push(9).unwrap();
        orderable_set_2.push(11).unwrap();

        let diference_vec = orderable_set_1.difference(&orderable_set_2).to_vec();

        assert!(diference_vec.len() == 2);
        assert!(diference_vec.contains(&1));
        assert!(diference_vec.contains(&3));
    }

    #[test]
    fn disjoin_test() {
        let mut orderable_set_1: OrderableSet<u8> = OrderableSet::new();

        orderable_set_1.push(1).unwrap();
        orderable_set_1.push(2).unwrap();
        orderable_set_1.push(9).unwrap();
        orderable_set_1.push(3).unwrap();

        let mut orderable_set_2: OrderableSet<u8> = OrderableSet::new();

        orderable_set_2.push(10).unwrap();
        orderable_set_2.push(2).unwrap();
        orderable_set_2.push(9).unwrap();
        orderable_set_2.push(11).unwrap();

        assert_eq!(orderable_set_1.is_disjoint(&orderable_set_2), false);
    }

    #[test]
    fn reverse_test() {
        let mut orderable_set: OrderableSet<u8> = OrderableSet::new();

        orderable_set.push(1).unwrap();
        orderable_set.push(2).unwrap();
        orderable_set.push(9).unwrap();
        orderable_set.push(3).unwrap();

        orderable_set.reverse();

        assert_eq!(orderable_set.to_vec(), [3, 9, 2, 1].to_vec());
    }
}
