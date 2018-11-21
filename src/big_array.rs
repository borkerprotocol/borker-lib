use std::fmt;
use std::marker::PhantomData;
use serde::ser::{Serialize, Serializer, SerializeTuple};
use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess, Error};

pub trait BigArray<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>;
}

macro_rules! big_array {
    ($($len:expr,)+) => {
        $(
            impl<'de, T> BigArray<'de> for [T; $len]
                where T: Default + Copy + Serialize + Deserialize<'de>
            {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer
                {
                    let mut seq = serializer.serialize_tuple(self.len())?;
                    for elem in &self[..] {
                        seq.serialize_element(elem)?;
                    }
                    seq.end()
                }

                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where D: Deserializer<'de>
                {
                    struct ArrayVisitor<T> {
                        element: PhantomData<T>,
                    }

                    impl<'de, T> Visitor<'de> for ArrayVisitor<T>
                        where T: Default + Copy + Deserialize<'de>
                    {
                        type Value = [T; $len];

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(concat!("an array of length ", $len))
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<[T; $len], A::Error>
                            where A: SeqAccess<'de>
                        {
                            let mut arr = [T::default(); $len];
                            for i in 0..$len {
                                arr[i] = seq.next_element()?
                                    .ok_or_else(|| Error::invalid_length(i, &self))?;
                            }
                            Ok(arr)
                        }
                    }

                    let visitor = ArrayVisitor { element: PhantomData };
                    deserializer.deserialize_tuple($len, visitor)
                }
            }

            impl<'de, T> BigArray<'de> for Option<[T; $len]>
                where T: Default + Copy + Serialize + Deserialize<'de>
            {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer
                {
                    match self {
                        Some(a) => {
                            let mut seq = serializer.serialize_tuple(a.len())?;
                            for elem in &a[..] {
                                seq.serialize_element(elem)?;
                            }
                            seq.end()
                        },
                        None => serializer.serialize_none(),
                    }
                }

                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where D: Deserializer<'de>
                {
                    struct ArrayVisitor<T> {
                        element: PhantomData<T>,
                    }

                    impl<'de, T> Visitor<'de> for ArrayVisitor<T>
                        where T: Default + Copy + Deserialize<'de>
                    {
                        type Value = Option<[T; $len]>;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(concat!("an array of length ", $len))
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                            where A: SeqAccess<'de>
                        {
                            let mut arr = [T::default(); $len];
                            for i in 0..$len {
                                arr[i] = seq.next_element()?
                                    .ok_or_else(|| Error::invalid_length(i, &self))?;
                            }
                            Ok(Some(arr))
                        }

                        fn visit_none<E>(self) -> Result<Self::Value, E>
                            where E: Error
                        {
                            Ok(None)
                        }

                        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                            where D: Deserializer<'de>
                        {
                            deserializer.deserialize_tuple($len, self)
                        }
                    }

                    let visitor = ArrayVisitor { element: PhantomData };
                    deserializer.deserialize_option(visitor)
                }
            }
        )+
    }
}

big_array! {
    40, 48, 50, 56, 64, 65, 72, 96, 100, 128, 160, 192, 200, 224, 256, 384, 512,
    768, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
}
