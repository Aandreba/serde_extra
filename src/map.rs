pub mod iter_map {
    use std::marker::PhantomData;
    use serde::{Serializer, Serialize, Deserializer, Deserialize, de::Visitor};

    #[inline]
    pub fn serialize<K: Serialize, V: Serialize, S: Serializer, I> (this: &I, ser: S) -> Result<S::Ok, S::Error> where for<'a> &'a I: IntoIterator<Item = &'a (K, V)> {
        return ser.collect_map(this.into_iter().map(|(k, v)| (k, v)));
    }

    #[inline]
    pub fn deserialize<'de, K: Deserialize<'de>, V: Deserialize<'de>, D: Deserializer<'de>, I: FromIterator<(K, V)>> (de: D) -> Result<I, D::Error> {
        struct LocalVisitor<K, V, I>(PhantomData<(K, V, I)>);

        impl<'de, K: Deserialize<'de>, V: Deserialize<'de>, I: FromIterator<(K, V)>> Visitor<'de> for LocalVisitor<K, V, I> {
            type Value = I;

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a map")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de>, {
                struct MapIter<'de, A: serde::de::MapAccess<'de>, K, V> (A, Option<A::Error>, PhantomData<(&'de (), K, V)>);
                impl<'de, A: serde::de::MapAccess<'de>, K: Deserialize<'de>, V: Deserialize<'de>> Iterator for MapIter<'de, A, K, V> {
                    type Item = (K, V);

                    #[inline]
                    fn next(&mut self) -> Option<Self::Item> {
                        return match self.0.next_entry() {
                            Ok(x) => x,
                            Err(e) => {
                                self.1 = Some(e);
                                None
                            }
                        }
                    }

                    #[inline]
                    fn size_hint(&self) -> (usize, Option<usize>) {
                        match self.0.size_hint() {
                            Some(x) => (x, Some(x)),
                            None => (0, None)
                        }
                    }
                }

                let mut iter = MapIter::<'de, A, K, V>(map, None, PhantomData);
                let result = I::from_iter(&mut iter);
                
                if let Some(e) = iter.1 {
                    return Err(e)
                }

                return Ok(result)
            }
        }
        
        de.deserialize_map(LocalVisitor(PhantomData))
    }
}