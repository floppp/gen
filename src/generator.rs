use rand::rngs::ThreadRng;
use rand::Rng;
use std::marker::PhantomData;

pub struct Gen<A, F>
where
    F: Fn(&mut ThreadRng) -> A,
{
    func: F,
    // _marker: PhantomData<A>,
}

impl<A, F> Gen<A, F>
where
    F: Fn(&mut ThreadRng) -> A,
{
    pub fn new(func: F) -> Gen<A, F> {
        Gen { func }
    }

    pub fn sample(&self, rng: &mut ThreadRng) -> A {
        (self.func)(rng)
    }

    pub fn map<B, G>(self, f: G) -> Gen<B, impl Fn(&mut ThreadRng) -> B>
    where
        G: Fn(A) -> B + Copy,
    {
        Gen::new(move |rng| f(self.sample(rng)))
    }

    pub fn filter<G>(self, predicate: G) -> Gen<A, impl Fn(&mut ThreadRng) -> A>
    where
        G: Fn(&A) -> bool + Copy,
    {
        Gen::new(move |rng| {
            let mut result;
            loop {
                result = self.sample(rng);
                if predicate(&result) {
                    return result;
                }
            }
        })
    }
}

impl<A, F> Gen<A, F>
where
    F: Fn(&mut ThreadRng) -> A + 'static,
{
    pub fn and_then<B, G, H>(self, f: G) -> Gen<B, impl Fn(&mut ThreadRng) -> B>
    where
        G: Fn(A) -> Gen<B, H> + 'static,
        H: Fn(&mut ThreadRng) -> B + 'static,
    {
        // nos ahorramos un par de indirecciones que supongo el compilador optimizaría
        // de todas formas.
        Gen::new(move |rng| {
            let a = (self.func)(rng);
            (f(a).func)(rng)
        })
        // Gen::new(move |rng| f(self.sample(rng)).sample(rng))
    }
}

// Si usamos `where F...` no es capaz al usarla de inferir tipos y por eso
// mejor hacer inline de la función en Gen en el tercer tipo de Gen
// que no usar el `where`.
impl<A, B> Gen<(A, B), fn(&mut ThreadRng) -> (A, B)> {
    pub fn gen_tuple(
        g: Gen<A, impl Fn(&mut ThreadRng) -> A>,
        h: Gen<B, impl Fn(&mut ThreadRng) -> B>,
    ) -> Gen<(A, B), impl Fn(&mut ThreadRng) -> (A, B)> {
        // Mucho mejor pero no puedo solucionar problema
        // al mover `h`.
        // g.and_then(move |a| h.map(move |b| (a, b)))
        Gen::new(move |rng| {
            let a = (g.func)(rng);
            let b = (h.func)(rng);
            (a, b)
        })
    }
}

impl<A, B, C> Gen<(A, B, C), fn(&mut ThreadRng) -> (A, B, C)> {
    pub fn gen_tuple3(
        g: Gen<A, impl Fn(&mut ThreadRng) -> A>,
        h: Gen<B, impl Fn(&mut ThreadRng) -> B>,
        i: Gen<C, impl Fn(&mut ThreadRng) -> C>,
    ) -> Gen<(A, B, C), impl Fn(&mut ThreadRng) -> (A, B, C)> {
        Gen::new(move |rng| {
            let a = (g.func)(rng);
            let b = (h.func)(rng);
            let c = (i.func)(rng);
            (a, b, c)
        })
    }
}

impl<A, B, C, D> Gen<(A, B, C, D), fn(&mut ThreadRng) -> (A, B, C, D)> {
    pub fn gen_tuple4(
        g: Gen<A, impl Fn(&mut ThreadRng) -> A>,
        h: Gen<B, impl Fn(&mut ThreadRng) -> B>,
        i: Gen<C, impl Fn(&mut ThreadRng) -> C>,
        j: Gen<D, impl Fn(&mut ThreadRng) -> D>,
    ) -> Gen<(A, B, C, D), impl Fn(&mut ThreadRng) -> (A, B, C, D)> {
        Gen::new(move |rng| {
            let a = (g.func)(rng);
            let b = (h.func)(rng);
            let c = (i.func)(rng);
            let d = (j.func)(rng);
            (a, b, c, d)
        })
    }
}

impl<A, B, C, D, E> Gen<(A, B, C, D, E), fn(&mut ThreadRng) -> (A, B, C, D, E)> {
    pub fn gen_tuple5(
        g: Gen<A, impl Fn(&mut ThreadRng) -> A>,
        h: Gen<B, impl Fn(&mut ThreadRng) -> B>,
        i: Gen<C, impl Fn(&mut ThreadRng) -> C>,
        j: Gen<D, impl Fn(&mut ThreadRng) -> D>,
        k: Gen<E, impl Fn(&mut ThreadRng) -> E>,
    ) -> Gen<(A, B, C, D, E), impl Fn(&mut ThreadRng) -> (A, B, C, D, E)> {
        Gen::new(move |rng| {
            let a = (g.func)(rng);
            let b = (h.func)(rng);
            let c = (i.func)(rng);
            let d = (j.func)(rng);
            let e = (k.func)(rng);
            (a, b, c, d, e)
        })
    }
}

impl Gen<bool, fn(&mut ThreadRng) -> bool> {
    pub fn gen_bool() -> Gen<bool, fn(&mut ThreadRng) -> bool> {
        Gen::new(|rng| rng.gen())
    }
}

impl Gen<i32, fn(&mut ThreadRng) -> i32> {
    pub fn gen_int() -> Gen<i32, fn(&mut ThreadRng) -> i32> {
        Gen::new(|rng| rng.gen())
    }

    pub fn gen_int_range(start: i32, end: i32) -> Gen<i32, impl Fn(&mut ThreadRng) -> i32> {
        Gen::new(move |rng| rng.gen_range(start..end))
    }
}
impl Gen<String, fn(&mut ThreadRng) -> String> {
    pub fn gen_string() -> Gen<String, impl Fn(&mut ThreadRng) -> String> {
        Gen::new(move |rng| {
            let size = Gen::gen_int_range(1, 100).sample(rng) as usize;
            Gen::gen_string_with_len(size).sample(rng)
        })
    }

    pub fn gen_alpha_lower() -> Gen<String, impl Fn(&mut ThreadRng) -> String> {
        Gen::new(move |rng| {
            let size = Gen::gen_int_range(1, 100).sample(rng) as usize;
            Gen::gen_alpha_lower_with_len(size).sample(rng)
        })
    }

    pub fn gen_string_with_len(n: usize) -> Gen<String, impl Fn(&mut ThreadRng) -> String> {
        Gen::gen_int_range(1, n as i32).map(|len| {
            let mut rng = rand::thread_rng();
            (0..len)
                .map(|_| (rng.gen_range(0..255) as u8) as char)
                .collect()
        })
    }

    pub fn gen_alpha_lower_with_len(n: usize) -> Gen<String, impl Fn(&mut ThreadRng) -> String> {
        Gen::gen_int_range(1, n as i32).map(|len| {
            let mut rng = rand::thread_rng();
            (0..len)
                .map(|_| (rng.gen_range(97..123) as u8) as char)
                .collect()
        })
    }
}

impl<A, F> Gen<A, F>
where
    F: Fn(&mut ThreadRng) -> A + 'static,
{
    pub fn list_of_n(
        n: usize,
        g: Gen<A, impl Fn(&mut ThreadRng) -> A>,
    ) -> Gen<Vec<A>, impl Fn(&mut ThreadRng) -> Vec<A>> {
        Gen::new(move |rng| (0..n).map(|_| g.sample(rng)).collect())
    }

    // Máximo mil elementos
    pub fn list_of(
        g: Gen<A, impl Fn(&mut ThreadRng) -> A>,
    ) -> Gen<Vec<A>, impl Fn(&mut ThreadRng) -> Vec<A>> {
        Gen::new(move |rng| {
            let n: i32 = rng.gen_range(0..1000);
            println!("n = {}", n);
            (0..n).map(|_| g.sample(rng)).collect()
        })
    }
}
