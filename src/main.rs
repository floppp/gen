mod fp_generator;
// mod state;

use crate::fp_generator::*;



// struct SimpleRGen {
// seed: u64,
// }

// impl SimpleRGen {
//     fn new(seed: u64) -> Self {
//         SimpleRGen { seed }
//     }
// }

// impl RGen for SimpleRGen {
//     fn next_int(&mut self) -> (i32, Box<dyn RGen>) {
//         let new_seed = (self.seed.wrapping_mul(0x5D588B656C078965) + 1) & 0xFFFFFFFF;
//         let new_rgen = Box::new(SimpleRGen::new(new_seed));
//         let random_number = (new_seed >> 16) as i32;

//         (random_number, new_rgen)
//     }
// }

fn main() {
    // RGen / SimmpleRGen
    // ========================
    let g = SimpleRGen::new();
    // let (r1, g1) = g.gen();
    // let (r2, g2) = g1.gen();
    // println!("{r1:}");
    // println!("{r2:}");

    // let ng = SimpleRGen::new(1988433333);
    // let (nr1, ng1) = ng.gen();
    // println!("{ng:}");
    // println!("{nr1:}");

    // State
    // ========================
    // let start_state = 0;
    // let state = State::new(&increment); // .map(&double);

    // let (_, new_state) = state.run(start_state);
    // println!("New state: {}", new_state);

    // // Creamos una instancia de State con una función simple
    // let initial_state = State::new(|s: i32| (s + 1, s));

    // // Usamos map para transformar el resultado entero en una cadena
    // // let mapped_state = initial_state.map(|a| format!("Formateo: {}", a));
    // let mapped_state = initial_state
    //     .and_then(&double_state)
    //     .map(&double_float)
    //     .map(|a| format!("Formateo: {}", a));

    // // Ejecutamos la función original y luego la función mapeada
    // let (result, new_state) = mapped_state.run(10);

    // println!("Resultado: {}, Nuevo Estado: {}", result, new_state);

    // Random
    // ========================
    // let mut rng = rand::thread_rng();
    // let rng = SimpleRGen::new_with_seed(1987);
    // let rng = SimpleRGen::new_with_seed(-1987);
    let rng = SimpleRGen::new();
    println!("{g}");
    println!("{rng}");
    println!("gen_bool:     {}", Gen::gen_bool().run(&rng).0);
    println!("gen_i8:      {}", Gen::gen_i8().run(&rng).0);
    println!("gen_i16:      {}", Gen::gen_i16().run(&rng).0);
    println!("gen_i32:      {}", Gen::gen_i32().run(&rng).0);
    println!("gen_i64:      {}", Gen::gen_i64().run(&rng).0);
    println!("gen_in_range: {}", Gen::gen_in_range(0, 100).run(&rng).0);
    println!(
        "gen_in_no_range: {}",
        Gen::gen_in_range(100, 100).run(&rng).0
    );
    println!("gen_f64:      {}", Gen::gen_f64().run(&rng).0);

    // let mut r = rng.clone();
    // for idx in 0..100 {
    //     let (a, s) = Gen::gen_i64().run(&r);
    //     let (b, t) = Gen::gen_i32().run(&s);
    //     r = t;
    //     println!("gen_i64:  {idx}    {a}      {b}");
    // }

    let string_gen = Gen::gen_string();
    let random_string = string_gen.run(&rng);
    println!("gen_string:      {}", random_string.0);

    let random_string_gen = Gen::gen_alpha_lower();
    let random_alpha_string = random_string_gen.run(&rng);
    println!("gen_alpha_lower: {}", random_alpha_string.0);

    // let int_gen = Gen::new(|rng| rng.gen_range(1, 9));
    // let squared_gen = int_gen.map(|x| x.0 * x.0);
    // let even_squared_gen = squared_gen.filter(|x| x % 2 == 1);
    // let random_odd_square = even_squared_gen.sample(&rng);
    // println!("Random odd square: {}", random_odd_square);

    let list_gen = Gen::list_of_n(5, Gen::gen_i64());
    println!("list_of_n: {:?}", list_gen.run(&rng).0);
    let tuple_gen = Gen::gen_tuple(Gen::gen_i64(), list_gen);
    println!("gen_tuple: {:?}", tuple_gen.run(&rng).0);

    let tuple_gen3 = Gen::gen_tuple3(Gen::gen_i64(), Gen::gen_bool(), Gen::gen_i64());
    println!("gen_tuple3: {:?}", tuple_gen3.run(&rng).0);

    let tuple_gen4 = Gen::gen_tuple4(Gen::gen_i64(), Gen::gen_bool(), tuple_gen3, Gen::gen_i64());
    let (a, s) = tuple_gen4.run(&rng);
    println!("gen_tuple4: {:?}", a);

    let tuple_gen5 = Gen::gen_tuple5(
        Gen::gen_i64(),
        Gen::gen_i64(),
        Gen::gen_bool(),
        tuple_gen4,
        Gen::gen_i64(),
    );
    // Usamos el último estado del anterior generador y partimos de él,
    // obteniendo valores distintos.
    println!("gen_tuple5: {:?}", tuple_gen5.run(&s).0);

    let hex_gen = Gen::gen_alpha_lower_16bits(5);
    let mut g = hex_gen.run(&s).1;
    println!("gen_alpha_lower_16bits: {:?}", hex_gen.run(&s).0);

    for _ in 0..10 {
        let (value, generator) = Gen::gen_random_uuid().run(&g);

        println!("{value}");
        g = generator;
    }

    let hm_gen = Gen::gen_flat_string_hashmap_random_values(10);
    println!(
        "gen_flat_string_hashmap_random_values: {:?}",
        hm_gen.run(&g).0
    );
}

// Intentos...

// trait RGen {
//     fn next_int(&mut self) -> (i32, Box<dyn RGen>); // State<Box<dyn RGen>, i32, Box<dyn Fn(Box<dyn RGen>) -> (i32, Box<dyn RGen>)>>;
// }

// struct SimpleRGen {
//     seed: u64,
// }

// impl SimpleRGen {
//     fn new(seed: u64) -> Self {
//         SimpleRGen { seed }
//     }
// }

// impl RGen for SimpleRGen {
//     fn next_int(&mut self) -> (i32, Box<dyn RGen>) {
//         let new_seed = (self.seed.wrapping_mul(0x5D588B656C078965) + 1) & 0xFFFFFFFF;
//         let new_rgen = Box::new(SimpleRGen::new(new_seed));
//         let random_number = (new_seed >> 16) as i32;

//         (random_number, new_rgen)
//     }
// }

// struct RandRngGen {
//     seed: u8,
//     gen: StdRng,
// }

// impl RandRngGen {
//     fn new(seed: u8) -> Self {
//         RandRngGen {
//             seed,
//             gen: StdRng::from_seed([seed; 32]),
//         }
//     }
// }

// impl RGen for RandRngGen {
//     fn next_int(&mut self) -> (i32, Box<dyn RGen>) {
//         // TODO: Mejorar esto, sacar 4 u64 aleatorios y de ahí sacar los 32 u8 y pasarlo como seed, no pasar un único valor.
//         let next_seed = (self.seed.wrapping_mul(0x5D) + 1) & 0xFF;
//         let next_rgen = Box::new(RandRngGen::new(next_seed));
//         let new_value = self.gen.gen();

//         (new_value, next_rgen)
//     }
// }

// impl dyn RGen {
//     fn gen_bool(g: &dyn RGen) -> (bool, Box<dyn RGen>) {
//         let (i, next_gen) = g.next_int();
//         (i > 0, next_gen)
//     }
// }
