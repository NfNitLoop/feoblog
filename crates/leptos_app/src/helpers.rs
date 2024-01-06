use leptos::SignalGet;

/// Helps avoid repetitive `move || x` code.
pub trait Getter<T> {
    fn getter(self) -> impl Fn() -> T;
}

impl <S: SignalGet + Clone + Copy> Getter<S::Value> for S {
    fn getter(self) -> impl Clone + Copy + Fn() -> S::Value {
        move || {
            self.get()
        }
    }
}
