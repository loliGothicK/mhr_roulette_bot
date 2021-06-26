use crate::concepts::SameAs;

pub trait TranslateTo<Target> {
    fn translate_to<T>(&self) -> anyhow::Result<Target>
    where
        T: SameAs<Target>;
}
