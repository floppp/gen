// Puntos importantes
// No puedo (no sé) usar trait porque al intentar usar Box<&dyn RGen> problemas porque en ciertos casos
// me obliga a definir como fn para capturar el tipo de lo que defino, pero entonces es demaiaso específico
/// y no me deja capturar  en el /closure/ las variables de la firma de la función (gen_in_range)
// Resta mucha flexibilidad, estoy atado a esta implementación. Cierto es que es muy fácil de modificar.
// Si defino la firma complciada como F (where F: ...) no es capaz de inferir el tipo
// Como en S => A, S ya digo qué es A, he de ser capaz de simplificar la firma de Gen y quitar la A
use std::collections::HashMap;
use std::fmt;

pub trait RGen {
    fn gen_int(&self) -> (i64, Box<dyn RGen>);
    fn gen_bool(&self) -> (bool, Box<dyn RGen>);
    fn gen_in_range(&self, begin: i64, end: i64) -> (i64, Box<dyn RGen>);
    // Puedes agregar más métodos según lo necesites
}

pub struct SimpleRGen {
    seed: i64,
}

impl Clone for SimpleRGen {
    fn clone(&self) -> Self {
        Self {
            seed: self.seed.clone(),
        }
    }
}

impl fmt::Display for SimpleRGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimpleRgen(seed:{})", self.seed)
    }
}

impl Default for SimpleRGen {
    fn default() -> Self {
        let seed =
            match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_nanos(),
                Err(_) => 0,
            } as i64;
        // Esto para hacer que parezca realmente un seed random.
        // Para hacerlo parecer aún más,
        //   let (a, s) = SimpleRGen::new_with_seed(seed).gen_int();
        //   let (_, t) = SimpleRGen::new_with_seed(a % seed).gen_int();
        //   t
        // y así de forma sucesiva. Podemos meter algo más de random usando gen_bool para
        // probar a hacer una cosa u otra, gen_in_range para limitar alguno, etc.
        let (_, s) = SimpleRGen::new_with_seed(seed).gen_i64();
        // println!("New seed: {seed}, new random {a}");
        s
    }
}

impl SimpleRGen {
    pub fn new_with_seed(seed: i64) -> Self {
        SimpleRGen { seed }
    }

    pub fn new() -> Self {
        SimpleRGen::default()
    }
}

impl SimpleRGen {
    fn gen_i8(&self) -> (i8, SimpleRGen) {
        let (i, r) = self.gen_i16();

        ((i >> 8) as i8, r)
    }

    fn gen_i16(&self) -> (i16, SimpleRGen) {
        let (i, r) = self.gen_i32();

        ((i >> 16) as i16, r)
    }

    // Generador del FP in Scala
    fn gen_i32(&self) -> (i32, SimpleRGen) {
        let new_seed =
            (self.seed.wrapping_mul(0x5DEECE66D as i64) + 0xB as i64) & 0xFFFFFFFFFFFF as i64;
        let new_rgen = SimpleRGen { seed: new_seed };
        let random_number = (new_seed >> 16) as i32;

        (random_number, new_rgen)
    }

    fn gen_u64(&self) -> (u64, SimpleRGen) {
        let new_seed = (self.seed.wrapping_mul(0x5D588B656C078965) + 1) & 0xFFFFFFFFFFFFFFF;
        let random_number = (new_seed >> 16) as u64;
        let r = SimpleRGen { seed: new_seed };

        (random_number, r)
    }

    fn gen_i64(&self) -> (i64, SimpleRGen) {
        let (u, _) = self.gen_u64();
        let (b, r) = self.gen_bool();
        let n = u as i64;

        (if b { n } else { -n }, r)
    }

    fn gen_bool(&self) -> (bool, SimpleRGen) {
        let (a, s) = self.gen_i32();

        (a % 2 == 0, s)
    }

    fn gen_in_range(&self, begin: i64, end: i64) -> (i64, SimpleRGen) {
        let (a, s) = self.gen_i64();
        let n = if a > 0 { a } else { -a };

        if begin == end {
            (begin, s)
        } else if begin > end {
            (begin + n % (begin - end), s)
        } else {
            (begin + n % (end - begin), s)
        }
    }
}

pub struct Gen<A, F>
where
    F: Fn(&SimpleRGen) -> (A, SimpleRGen),
{
    func: F,
    // _marker: PhantomData<A>,
}

impl<A, F> Gen<A, F>
where
    F: Fn(&SimpleRGen) -> (A, SimpleRGen),
{
    pub fn new(func: F) -> Gen<A, F> {
        Gen { func }
    }
    pub fn run(&self, rng: &SimpleRGen) -> (A, SimpleRGen) {
        (self.func)(rng)
    }

    pub fn sample(&self, rng: &SimpleRGen) -> A {
        self.run(rng).0
    }

    pub fn map<B, G>(self, f: G) -> Gen<B, impl Fn(&SimpleRGen) -> (B, SimpleRGen)>
    where
        G: Fn(A) -> B + Copy,
    {
        Gen::new(move |rng| {
            let (a, s) = self.run(rng);
            let b = f(a);
            (b, s)
        })
    }

    pub fn filter<G>(self, predicate: G) -> Gen<A, impl Fn(&SimpleRGen) -> (A, SimpleRGen)>
    where
        G: Fn(&A) -> bool + Copy,
    {
        Gen::new(move |rng| {
            let mut result;
            let mut g = rng.clone();
            loop {
                result = self.run(&g);
                let (a, s) = result;
                if predicate(&a) {
                    return (a, s);
                }
                g = s;
            }
        })
    }
}

impl<A, F> Gen<A, F>
where
    F: Fn(&SimpleRGen) -> (A, SimpleRGen),
{
    pub fn and_then<B, G, H>(self, f: G) -> Gen<B, impl Fn(&SimpleRGen) -> (B, SimpleRGen)>
    where
        G: Fn(A) -> Gen<B, H> + 'static,
        H: Fn(&SimpleRGen) -> (B, SimpleRGen) + 'static,
    {
        // nos ahorramos un par de indirecciones que supongo el compilador optimizaría
        // de todas formas.
        Gen::new(move |rng| {
            let (a, _) = (self.func)(rng);
            (f(a).func)(rng)
        })
        // Gen::new(move |rng| f(self.sample(rng)).sample(rng))
    }
}

// Si usamos `where F...` no es capaz al usarla de inferir tipos y por eso
// mejor hacer inline de la función en Gen en el tercer tipo de Gen
// que no usar el `where`.
impl<A, B> Gen<(A, B), fn(&SimpleRGen) -> ((A, B), SimpleRGen)> {
    pub fn gen_tuple(
        g: Gen<A, impl Fn(&SimpleRGen) -> (A, SimpleRGen)>,
        h: Gen<B, impl Fn(&SimpleRGen) -> (B, SimpleRGen)>,
    ) -> Gen<(A, B), impl Fn(&SimpleRGen) -> ((A, B), SimpleRGen)> {
        // Mucho mejor pero no puedo solucionar problema
        // al mover `h`.
        // g.and_then(move |a| h.map(move |b| (a, b)))
        Gen::new(move |rng| {
            let (a, s) = (g.func)(rng);
            let (b, t) = (h.func)(&s);
            ((a, b), t)
        })
    }
}

impl<A, B, C> Gen<(A, B, C), fn(&SimpleRGen) -> ((A, B, C), SimpleRGen)> {
    pub fn gen_tuple3(
        g: Gen<A, impl Fn(&SimpleRGen) -> (A, SimpleRGen)>,
        h: Gen<B, impl Fn(&SimpleRGen) -> (B, SimpleRGen)>,
        j: Gen<C, impl Fn(&SimpleRGen) -> (C, SimpleRGen)>,
    ) -> Gen<(A, B, C), impl Fn(&SimpleRGen) -> ((A, B, C), SimpleRGen)> {
        // Mucho mejor pero no puedo solucionar problema
        // al mover `h`.
        // g.and_then(move |a| h.map(move |b| (a, b)))
        Gen::new(move |rng| {
            let (a, s) = (g.func)(rng);
            let (b, t) = (h.func)(&s);
            let (c, u) = (j.func)(&t);
            ((a, b, c), u)
        })
    }
}

impl<A, B, C, D> Gen<(A, B, C, D), fn(&SimpleRGen) -> ((A, B, C, D), SimpleRGen)> {
    pub fn gen_tuple4(
        g: Gen<A, impl Fn(&SimpleRGen) -> (A, SimpleRGen)>,
        h: Gen<B, impl Fn(&SimpleRGen) -> (B, SimpleRGen)>,
        j: Gen<C, impl Fn(&SimpleRGen) -> (C, SimpleRGen)>,
        k: Gen<D, impl Fn(&SimpleRGen) -> (D, SimpleRGen)>,
    ) -> Gen<(A, B, C, D), impl Fn(&SimpleRGen) -> ((A, B, C, D), SimpleRGen)> {
        // Mucho mejor pero no puedo solucionar problema
        // al mover `h`.
        // g.and_then(move |a| h.map(move |b| (a, b)))
        Gen::new(move |rng| {
            let (a, s) = (g.func)(rng);
            let (b, t) = (h.func)(&s);
            let (c, u) = (j.func)(&t);
            let (d, v) = (k.func)(&u);
            ((a, b, c, d), v)
        })
    }
}

impl<A, B, C, D, E> Gen<(A, B, C, D, E), fn(&SimpleRGen) -> ((A, B, C, D, E), SimpleRGen)> {
    pub fn gen_tuple5(
        g: Gen<A, impl Fn(&SimpleRGen) -> (A, SimpleRGen)>,
        h: Gen<B, impl Fn(&SimpleRGen) -> (B, SimpleRGen)>,
        j: Gen<C, impl Fn(&SimpleRGen) -> (C, SimpleRGen)>,
        k: Gen<D, impl Fn(&SimpleRGen) -> (D, SimpleRGen)>,
        l: Gen<E, impl Fn(&SimpleRGen) -> (E, SimpleRGen)>,
    ) -> Gen<(A, B, C, D, E), impl Fn(&SimpleRGen) -> ((A, B, C, D, E), SimpleRGen)> {
        Gen::new(move |rng| {
            let (a, s) = (g.func)(rng);
            let (b, t) = (h.func)(&s);
            let (c, u) = (j.func)(&t);
            let (d, v) = (k.func)(&u);
            let (e, w) = (l.func)(&v);
            ((a, b, c, d, e), w)
        })
    }
}

impl Gen<bool, fn(&SimpleRGen) -> (bool, SimpleRGen)> {
    pub fn gen_bool() -> Gen<bool, fn(&SimpleRGen) -> (bool, SimpleRGen)> {
        Gen::new(|rng| rng.gen_bool())
    }
}

impl Gen<i8, fn(&SimpleRGen) -> (i8, SimpleRGen)> {
    pub fn gen_i8() -> Gen<i8, fn(&SimpleRGen) -> (i8, SimpleRGen)> {
        Gen::new(|rng| rng.gen_i8())
    }
}

impl Gen<i16, fn(&SimpleRGen) -> (i16, SimpleRGen)> {
    pub fn gen_i16() -> Gen<i16, fn(&SimpleRGen) -> (i16, SimpleRGen)> {
        Gen::new(|rng| rng.gen_i16())
    }
}

impl Gen<i32, fn(&SimpleRGen) -> (i32, SimpleRGen)> {
    pub fn gen_i32() -> Gen<i32, fn(&SimpleRGen) -> (i32, SimpleRGen)> {
        Gen::new(|rng| rng.gen_i32())
    }
}

impl Gen<f64, fn(&SimpleRGen) -> (f64, SimpleRGen)> {
    // Rango (0, 1). Si queremos rango (-1, 1) sería generando i64.
    pub fn gen_f64() -> Gen<f64, fn(&SimpleRGen) -> (f64, SimpleRGen)> {
        Gen::new(|rng| {
            let (i1, r) = rng.gen_u64();
            let (i2, t) = r.gen_u64();
            let a = i1 as f64 / i2 as f64;

            if a < 1f64 {
                (a, t)
            } else {
                (1f64 / a, t)
            }
        })
    }
}

impl Gen<i64, fn(&SimpleRGen) -> (i64, SimpleRGen)> {
    pub fn gen_i64() -> Gen<i64, fn(&SimpleRGen) -> (i64, SimpleRGen)> {
        Gen::new(|rng| rng.gen_i64())
    }

    pub fn gen_in_range(
        start: i64,
        end: i64,
    ) -> Gen<i64, impl Fn(&SimpleRGen) -> (i64, SimpleRGen)> {
        Gen::new(move |rng| rng.gen_in_range(start, end))
    }
}

impl Gen<String, fn(&SimpleRGen) -> (String, SimpleRGen)> {
    pub fn gen_string_with_max_len(
        max_len: usize,
    ) -> Gen<String, impl Fn(&SimpleRGen) -> (String, SimpleRGen)> {
        // Podría hacerlo con filter también, llamando gen_string
        // y filtrando si len > max_len, pero creo que bastante más costoso,
        // mayor cuanto menor longitud de cadena.
        Gen::new(move |rng| {
            let (len, n_rng) = Gen::gen_in_range(0, max_len as i64).run(rng);
            Gen::gen_string_with_len(len as usize).run(&n_rng)
        })
    }

    pub fn gen_string_with_len(
        len: usize,
    ) -> Gen<String, impl Fn(&SimpleRGen) -> (String, SimpleRGen)> {
        Gen::new(move |rng| {
            let mut acc = String::default();
            let mut t: Option<SimpleRGen> = None;
            for idx in 0..len {
                // println!("{idx}");
                let (a, ri) = match t {
                    Some(t) => t.gen_in_range(0, 255),
                    _ => rng.gen_in_range(0, 255),
                };
                t = Some(ri);
                let c = a as u8 as char;
                acc.push(c);
            }

            (acc, t.unwrap_or_default())
        })
    }

    pub fn gen_string() -> Gen<String, fn(&SimpleRGen) -> (String, SimpleRGen)> {
        Gen::new(move |rng| {
            let (a, s) = Gen::gen_in_range(1, 100).run(rng);
            println!("max len {a}");
            Gen::gen_string_with_len(a as usize).run(&s)
        })
    }

    pub fn gen_alpha_lower_16bits(
        len: usize,
    ) -> Gen<String, impl Fn(&SimpleRGen) -> (String, SimpleRGen)> {
        Gen::new(move |rng| {
            let mut acc: Vec<u8> = Vec::new();
            let mut t: Option<SimpleRGen> = None;

            for _ in 0..len {
                let (a, ri) = match t {
                    Some(t) => t.gen_in_range(97, 103),
                    _ => rng.gen_in_range(97, 103),
                };
                let (b, rj) = ri.gen_in_range(48, 58);
                let (alpha1, rk) = rj.gen_bool();
                // porque hay 10 números y 6 letras, para que no haya
                // tantas letras hago doble comprobación. Tampoco es
                // exacto pero no es tan cantoso en cuanto al número
                // de letras.
                if alpha1 {
                    let (alpha2, rl) = rk.gen_bool();
                    let c = if alpha2 { a as u8 } else { b as u8 };
                    acc.push(c);
                    t = Some(rl);
                } else {
                    acc.push(b as u8);
                    t = Some(rk);
                }
            }

            (
                String::from_utf8(acc).unwrap_or_default(),
                t.unwrap_or_default(),
            )
        })
    }

    // al azar, no cumple con nada más que con las longitudes de las subcadenas
    pub fn gen_random_uuid() -> Gen<String, impl Fn(&SimpleRGen) -> (String, SimpleRGen)> {
        Gen::new(move |rng| {
            let (fst, r1) = Gen::gen_alpha_lower_16bits(8).run(rng);
            let (scd, r2) = Gen::gen_alpha_lower_16bits(4).run(&r1);
            let (thd, r3) = Gen::gen_alpha_lower_16bits(4).run(&r2);
            let (fth, r4) = Gen::gen_alpha_lower_16bits(4).run(&r3);
            let (fif, r5) = Gen::gen_alpha_lower_16bits(12).run(&r4);

            (format!("{fst}-{scd}-{thd}-{fth}-{fif}"), r5)
        })
    }

    pub fn gen_alpha_lower_with_max_len(
        max_len: usize,
    ) -> Gen<String, impl Fn(&SimpleRGen) -> (String, SimpleRGen)> {
        // Podría hacerlo con filter también, llamando gen_string
        // y filtrando si len > max_len, pero creo que bastante más costoso,
        // mayor cuanto menor longitud de cadena.
        Gen::new(move |rng| {
            let (len, n_rng) = Gen::gen_in_range(0, max_len as i64).run(rng);
            Gen::gen_alpha_lower_with_len(len as usize).run(&n_rng)
        })
    }

    // Simplicar las dos que usan rango, que son iguales
    pub fn gen_alpha_lower_with_len(
        len: usize,
    ) -> Gen<String, impl Fn(&SimpleRGen) -> (String, SimpleRGen)> {
        Gen::new(move |rng| {
            let mut acc = String::default();
            let mut t: Option<SimpleRGen> = None;
            for _ in 0..len {
                let (a, ri) = match t {
                    Some(t) => t.gen_in_range(97, 123),
                    _ => rng.gen_in_range(97, 123),
                };
                t = Some(ri);
                let c = a as u8 as char;
                acc.push(c);
            }

            (acc, t.unwrap_or_default())
        })
    }

    pub fn gen_alpha_lower() -> Gen<String, fn(&SimpleRGen) -> (String, SimpleRGen)> {
        Gen::new(move |rng| {
            let (a, s) = Gen::gen_in_range(1, 100).run(rng);
            Gen::gen_alpha_lower_with_len(a as usize).run(&s)
        })
    }
}

impl<A, F> Gen<A, F>
where
    F: Fn(&SimpleRGen) -> (A, SimpleRGen) + 'static,
{
    pub fn list_of_n(
        len: usize,
        // g: Gen<A, impl fn(&dyn RGen) -> A>,
        g: Gen<A, F>, //impl Fn(&SimpleRGen) -> (A, SimpleRGen)>,
    ) -> Gen<Vec<A>, impl Fn(&SimpleRGen) -> (Vec<A>, SimpleRGen)> {
        Gen::new(move |rng| {
            let mut acc = Vec::<A>::new();
            let mut t: Option<SimpleRGen> = None;
            for _ in 0..len {
                let (a, ri) = match t {
                    Some(t) => g.run(&t),
                    _ => g.run(rng),
                };
                t = Some(ri);
                acc.push(a);
            }

            (acc, t.unwrap())
        })
        // Gen::new(move |rng| (0..n).map(|_| g.sample(rng)).collect())
    }

    // Máximo mil elementos
    pub fn list_of(
        // g: Gen<A, impl Fn(&SimpleRGen) -> (A, SimpleRGen)>,
        g: Gen<A, F>,
    ) -> Gen<Vec<A>, impl Fn(&SimpleRGen) -> (Vec<A>, SimpleRGen)> {
        Gen::new(move |rng| {
            let (a, s) = rng.gen_in_range(0, 1000);
            let mut acc = Vec::<A>::new();
            let mut t: Option<SimpleRGen> = None;
            for _ in 0..a {
                let (a, ri) = match t {
                    Some(t) => g.run(&t),
                    _ => g.run(rng),
                };
                t = Some(ri);
                acc.push(a);
            }

            (acc, t.unwrap())
        })
    }

    //     pub fn list_of(
    //         g: Gen<A, impl Fn(&SimpleRGen) -> (A, SimpleRGen)>,
    //     ) -> Gen<Vec<A>, impl Fn(&SimpleRGen) -> (Vec<A>, SimpleRGen)> {
    //         Gen::new(move |rng| {
    //             let (a, s) = rng.gen_in_range(0, 1000);
    //             let mut acc = Vec::<A>::new();
    //             let mut t: Option<SimpleRGen> = None;
    //             for _ in 0..a {
    //                 let (a, ri) = match t {
    //                     Some(t) => g.run(&t),
    //                     _ => g.run(rng),
    //                 };
    //                 t = Some(ri);
    //                 acc.push(a);
    //             }

    //             (acc, t.unwrap())
    //         })
    //     }
}

// // Implementación muy básica para probar rápido en ASAPI.
impl Gen<HashMap<String, String>, fn(&SimpleRGen) -> (HashMap<String, String>, SimpleRGen)> {
    pub fn gen_flat_string_hashmap_random_values(
        n_keys: usize,
    ) -> Gen<HashMap<String, String>, impl Fn(&SimpleRGen) -> (HashMap<String, String>, SimpleRGen)>
    {
        Gen::new(move |rng| {
            let mut hm: HashMap<String, String> = HashMap::new();
            let mut g = SimpleRGen { seed: rng.seed };

            for _ in 0..n_keys {
                let (key, g1) = Gen::gen_alpha_lower_with_max_len(5).run(&g);
                let (alpha, g2) = Gen::gen_bool().run(&g1);
                if alpha {
                    let (value, g3) = Gen::gen_i64().run(&g2);
                    g = g3;
                    hm.insert(key, value.to_string());
                } else {
                    let (value, g3) = Gen::gen_string_with_max_len(10).run(&g2);
                    g = g3;
                    hm.insert(key, value);
                };
            }

            (hm, g)
        })
    }
}
