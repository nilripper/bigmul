use plotters::prelude::*;
use rand::Rng;
use std::cmp;
use std::time::Instant;

#[derive(Clone, Debug)]
struct BigInt {
    digits: Vec<u32>,
}

const BASE: u64 = 1_000_000_000;

impl BigInt {
    fn new() -> Self {
        BigInt { digits: vec![0] }
    }

    fn from_str(s: &str) -> Self {
        if s.is_empty() {
            return BigInt::new();
        }
        let mut digits = Vec::new();
        let mut ss = s.to_string();
        while !ss.is_empty() {
            let chunk_size = cmp::min(9, ss.len());
            let chunk = &ss[ss.len() - chunk_size..];
            let digit = chunk.parse::<u32>().unwrap_or(0);
            digits.push(digit);
            ss.truncate(ss.len() - chunk_size);
        }
        BigInt::normalize(&mut digits);
        if digits.is_empty() {
            digits.push(0);
        }
        BigInt { digits }
    }

    fn normalize(digits: &mut Vec<u32>) {
        while digits.len() > 1 && *digits.last().unwrap() == 0 {
            digits.pop();
        }
    }

    fn to_string(&self) -> String {
        if self.digits.is_empty() || self.digits == vec![0] {
            return "0".to_string();
        }
        let mut s = self.digits.last().unwrap().to_string();
        for &d in self.digits.iter().rev().skip(1) {
            s.push_str(&format!("{:09}", d));
        }
        s
    }

    fn add(&self, other: &BigInt) -> BigInt {
        BigInt {
            digits: Self::add_slices(&self.digits, &other.digits),
        }
    }

    fn add_slices(a: &[u32], b: &[u32]) -> Vec<u32> {
        let max_len = cmp::max(a.len(), b.len());
        let mut result = vec![0u32; max_len + 1];
        let mut carry: u64 = 0;
        for i in 0..max_len {
            let ai = if i < a.len() { a[i] as u64 } else { 0 };
            let bi = if i < b.len() { b[i] as u64 } else { 0 };
            let sum = ai + bi + carry;
            result[i] = (sum % BASE) as u32;
            carry = sum / BASE;
        }
        if carry > 0 {
            result[max_len] = carry as u32;
        }
        Self::normalize(&mut result);
        result
    }

    fn sub(&self, other: &BigInt) -> BigInt {
        BigInt {
            digits: Self::sub_slices(&self.digits, &other.digits),
        }
    }

    fn sub_slices(a: &[u32], b: &[u32]) -> Vec<u32> {
        let max_len = a.len();
        let mut result = vec![0u32; max_len];
        let mut borrow: i64 = 0;
        for i in 0..max_len {
            let ai = a[i] as i64;
            let bi = if i < b.len() { b[i] as i64 } else { 0 };
            let mut diff = ai - bi - borrow;
            if diff < 0 {
                diff += BASE as i64;
                borrow = 1;
            } else {
                borrow = 0;
            }
            result[i] = diff as u32;
        }
        Self::normalize(&mut result);
        result
    }

    fn shift_left(&self, k: usize) -> BigInt {
        BigInt {
            digits: Self::shift_left_slices(&self.digits, k),
        }
    }

    fn shift_left_slices(digits: &[u32], k: usize) -> Vec<u32> {
        if digits == &[0] {
            return vec![0];
        }
        let mut res = vec![0u32; k + digits.len()];
        res[k..].copy_from_slice(digits);
        res
    }

    fn mul_direct(&self, other: &BigInt) -> BigInt {
        BigInt {
            digits: Self::mul_direct_slices(&self.digits, &other.digits),
        }
    }

    fn mul_direct_slices(a: &[u32], b: &[u32]) -> Vec<u32> {
        let len_a = a.len();
        let len_b = b.len();
        if len_a == 0 || len_b == 0 {
            return vec![0];
        }
        let mut result = vec![0u32; len_a + len_b];
        for i in 0..len_a {
            let mut carry: u64 = 0;
            for j in 0..len_b {
                let temp = a[i] as u64 * b[j] as u64 + result[i + j] as u64 + carry;
                result[i + j] = (temp % BASE) as u32;
                carry = temp / BASE;
            }
            let mut k = i + len_b;
            while carry > 0 {
                if k == result.len() {
                    result.push(0);
                }
                let temp = result[k] as u64 + carry;
                result[k] = (temp % BASE) as u32;
                carry = temp / BASE;
                k += 1;
            }
        }
        Self::normalize(&mut result);
        result
    }

    fn mul_dc(&self, other: &BigInt) -> BigInt {
        BigInt {
            digits: Self::mul_dc_slices(&self.digits, &other.digits),
        }
    }

    fn mul_dc_slices(a: &[u32], b: &[u32]) -> Vec<u32> {
        if a.is_empty() || b.is_empty() {
            return vec![0];
        }
        let n = cmp::max(a.len(), b.len());
        if n <= 32 {
            return Self::mul_direct_slices(a, b);
        }
        let m = n / 2;
        let a0 = &a[0..cmp::min(m, a.len())];
        let a1 = if a.len() > m { &a[m..] } else { &[] };
        let b0 = &b[0..cmp::min(m, b.len())];
        let b1 = if b.len() > m { &b[m..] } else { &[] };
        let p = Self::mul_dc_slices(a0, b0);
        let q = Self::mul_dc_slices(a1, b1);
        let r = Self::mul_dc_slices(a0, b1);
        let s = Self::mul_dc_slices(a1, b0);
        let mid = Self::add_slices(&r, &s);
        let q_shifted = Self::shift_left_slices(&q, 2 * m);
        let mid_shifted = Self::shift_left_slices(&mid, m);
        let temp = Self::add_slices(&q_shifted, &mid_shifted);
        Self::add_slices(&temp, &p)
    }

    fn mul_karatsuba(&self, other: &BigInt) -> BigInt {
        BigInt {
            digits: Self::mul_karatsuba_slices(&self.digits, &other.digits),
        }
    }

    fn mul_karatsuba_slices(a: &[u32], b: &[u32]) -> Vec<u32> {
        if a.is_empty() || b.is_empty() {
            return vec![0];
        }
        let n = cmp::max(a.len(), b.len());
        if n <= 32 {
            return Self::mul_direct_slices(a, b);
        }
        let m = n / 2;
        let a0 = &a[0..cmp::min(m, a.len())];
        let a1 = if a.len() > m { &a[m..] } else { &[] };
        let b0 = &b[0..cmp::min(m, b.len())];
        let b1 = if b.len() > m { &b[m..] } else { &[] };
        let p = Self::mul_karatsuba_slices(a0, b0);
        let q = Self::mul_karatsuba_slices(a1, b1);
        let sum_a = Self::add_slices(a0, a1);
        let sum_b = Self::add_slices(b0, b1);
        let u = Self::mul_karatsuba_slices(&sum_a, &sum_b);
        let sum_pq = Self::add_slices(&p, &q);
        let mid = Self::sub_slices(&u, &sum_pq);
        let q_shifted = Self::shift_left_slices(&q, 2 * m);
        let mid_shifted = Self::shift_left_slices(&mid, m);
        let temp = Self::add_slices(&q_shifted, &mid_shifted);
        Self::add_slices(&temp, &p)
    }
}

impl PartialEq for BigInt {
    fn eq(&self, other: &Self) -> bool {
        self.digits == other.digits
    }
}

fn random_bigint(d: usize) -> BigInt {
    if d == 0 {
        return BigInt::new();
    }
    let mut rng = rand::thread_rng();
    let mut s = rng.gen_range(1..=9).to_string();
    for _ in 1..d {
        s.push_str(&rng.gen_range(0..=9).to_string());
    }
    BigInt::from_str(&s)
}

fn main() {
    let min_d: usize = 1000;
    let max_d: usize = 10000;
    let num_sizes: usize = 100;
    let step = (max_d - min_d) / (num_sizes - 1);
    let mut ns: Vec<usize> = (0..num_sizes).map(|i| min_d + i * step).collect();

    if ns.last().unwrap() != &max_d {
        *ns.last_mut().unwrap() = max_d;
    }

    let num_instances = 10;

    let mut avgs_direct: Vec<f64> = Vec::with_capacity(num_sizes);
    let mut avgs_dc: Vec<f64> = Vec::with_capacity(num_sizes);
    let mut avgs_kara: Vec<f64> = Vec::with_capacity(num_sizes);

    for &n in &ns {
        let mut times_direct = 0.0;
        let mut times_dc = 0.0;
        let mut times_kara = 0.0;
        for _ in 0..num_instances {
            let a = random_bigint(n);
            let b = random_bigint(n);

            let start = Instant::now();
            let prod1 = a.mul_direct(&b);
            times_direct += start.elapsed().as_secs_f64();

            let start = Instant::now();
            let prod2 = a.mul_dc(&b);
            times_dc += start.elapsed().as_secs_f64();

            let start = Instant::now();
            let prod3 = a.mul_karatsuba(&b);
            times_kara += start.elapsed().as_secs_f64();

            assert_eq!(prod1, prod2);
            assert_eq!(prod1, prod3);
        }
        avgs_direct.push(times_direct / num_instances as f64);
        avgs_dc.push(times_dc / num_instances as f64);
        avgs_kara.push(times_kara / num_instances as f64);
    }

    // Print data
    for i in 0..ns.len() {
        println!(
            "n={}, direct={:.6}, dc={:.6}, kara={:.6}",
            ns[i], avgs_direct[i], avgs_dc[i], avgs_kara[i]
        );
    }

    // Plot graph
    std::fs::create_dir_all("./assets").expect("Failed to create ./assets directory");
    let root = BitMapBackend::new("./assets/multiplication_times.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let max_time = avgs_direct
        .iter()
        .chain(avgs_dc.iter())
        .chain(avgs_kara.iter())
        .fold(f64::MIN, |m, &v| m.max(v));
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Multiplication Algorithms Comparison",
            ("sans-serif", 50).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(
            ns[0] as f32..*ns.last().unwrap() as f32 + 1.0,
            0f32..(max_time * 1.1) as f32,
        )
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Input Size (number of digits)")
        .y_desc("Average Execution Time (seconds)")
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            ns.iter()
                .zip(avgs_direct.iter())
                .map(|(&x, &y)| (x as f32, y as f32)),
            &RED,
        ))
        .unwrap()
        .label("Direct Multiplication")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .draw_series(LineSeries::new(
            ns.iter()
                .zip(avgs_dc.iter())
                .map(|(&x, &y)| (x as f32, y as f32)),
            &GREEN,
        ))
        .unwrap()
        .label("Simple Divide & Conquer")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

    chart
        .draw_series(LineSeries::new(
            ns.iter()
                .zip(avgs_kara.iter())
                .map(|(&x, &y)| (x as f32, y as f32)),
            &BLUE,
        ))
        .unwrap()
        .label("Karatsuba")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();

    root.present().unwrap();

    println!("Graph saved to ./assets/multiplication_times.png");
}
