pub fn consecutive_pairs<T>(iter: impl Iterator<Item = T> + Clone) -> impl Iterator<Item = (T, T)> {
    let cloned = iter.clone();
    iter.zip(cloned.skip(1))
}
