use std::marker::PhantomData;

pub struct State<S, A, F: Fn(S) -> (A, S)> {
    func: F,
    _marker: PhantomData<(S, A)>,
}

impl<S, A, F> State<S, A, F>
where
    F: Fn(S) -> (A, S),
{
    pub fn new(func: F) -> State<S, A, F> {
        State {
            func,
            _marker: PhantomData,
        }
    }

    pub fn run(&self, s: S) -> (A, S) {
        (self.func)(s)
    }

    pub fn map<B, G>(self, f: G) -> State<S, B, impl Fn(S) -> (B, S)>
    where
        G: Fn(A) -> B + 'static, // la función que mapea no mantiene referencias a variables locales que puedan invalidarse, para eso gastamos `'static` aquí
    {
        State::new(move |s| {
            let (a, new_st) = self.run(s);
            let b = f(a);
            (b, new_st)
        })
    }

    pub fn and_then<B, G>(self, f: G) -> State<S, B, impl Fn(S) -> (B, S)>
    where
        G: Fn(A) -> (B, S) + 'static,
    {
        State::new(move |s| {
            let (a, _) = self.run(s);
            let (b, s) = f(a);
            (b, s)
        })
    }
}

pub fn increment(s: i32) -> ((), i32) {
    ((), s + 1)
}

pub fn double_float(s: f32) -> f32 {
    s * 2.1
}

pub fn double_state(s: i32) -> (f32, i32) {
    ((s as f32) * 2.1, s)
}
