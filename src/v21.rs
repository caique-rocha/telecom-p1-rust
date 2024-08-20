use std::{f32::consts::PI, ops::Rem};

pub struct V21RX {
    sampling_period: f32,
    samples_per_symbol: usize,
    omega_mark: f32,
    omega_space: f32,
    pointer: usize,
    buffer_index: Vec<f32>,
    v0r: f32,
    v0i: f32,
    v1r: f32,
    v1i: f32,
    last_decision: f32
}

impl V21RX {
    pub fn new(
        sampling_period: f32,
        samples_per_symbol: usize,
        omega_mark: f32,
        omega_space: f32,
    ) -> Self {
        let buffer_index = vec![0.0; samples_per_symbol * 2];
        Self {
            sampling_period,
            samples_per_symbol,
            omega_mark,
            omega_space,
            buffer_index,
            pointer: 0,
            v0r: 0.0,
            v0i: 0.0,
            v1r: 0.0,
            v1i: 0.0,
            last_decision: 0.0,
        }
    }

    pub fn demodulate(&mut self, in_samples: &[f32], out_samples: &mut [u8]) {

        let r: f32 = 0.999;
        let l = self.samples_per_symbol as f32;
        let t = self.sampling_period as f32;
        let omega0 = self.omega_space;
        let omega1 = self.omega_mark;
        // Coeficiente para o filtro
        let alpha = 2.0 * std::f32::consts::PI * t * 300.0 / (2.0 * std::f32::consts::PI * t * 300.0 + 1.0);

        for (i, &sample) in in_samples.iter().enumerate() {
            self.pointer = (self.pointer + 1) % self.buffer_index.len();
            self.buffer_index[self.pointer] = sample;

            // Índice para amostras anteriores baseado no comprimento do símbolo
            let last_symbol = (self.pointer + self.buffer_index.len() - self.samples_per_symbol) % self.buffer_index.len();

            // Calcula as partes real (r) e imaginária (i) para ambas as frequências
            let v0r = sample
                - r.powf(l) * (omega0 * l * t).cos() * self.buffer_index[last_symbol]
                + r * (omega0 * t).cos() * self.v0r
                - r * (omega0 * t).sin() * self.v0i;
            let v0i = -r.powf(l) * (omega0 * l * t).sin() * self.buffer_index[last_symbol]
                + r * (omega0 * t).cos() * self.v0i
                + r * (omega0 * t).sin() * self.v0r;

            let v1r = sample
                - r.powf(l) * (omega1 * l * t).cos() * self.buffer_index[last_symbol]
                + r * (omega1 * t).cos() * self.v1r
                - r * (omega1 * t).sin() * self.v1i;
            let v1i = -r.powf(l) * (omega1 * l * t).sin() * self.buffer_index[last_symbol]
                + r * (omega1 * t).cos() * self.v1i
                + r * (omega1 * t).sin() * self.v1r;

            // Variável de decisão baseada na diferença de potência entre os dois tons
            let decision = v1r * v1r + v1i * v1i - v0r * v0r - v0i * v0i;

            // Aplica um filtro passa-baixas simples para estabilidade
            let filtered_decision = self.last_decision + alpha * (decision - self.last_decision);
            self.last_decision = filtered_decision;

            // Armazena os resultados
            out_samples[i] = if filtered_decision > 0.0 || filtered_decision.abs() < 180.0 { 1 } else { 0 };


            self.v0r = v0r;
            self.v0i = v0i;
            self.v1r = v1r;
            self.v1i = v1i;
        }
    }

}

pub struct V21TX {
    sampling_period: f32,
    omega_mark: f32,
    omega_space: f32,
    phase: f32,
}

impl V21TX {
    pub fn new(sampling_period: f32, omega_mark: f32, omega_space: f32) -> Self {
        Self {
            sampling_period,
            omega_mark,
            omega_space,
            phase: 0.,
        }
    }

    pub fn modulate(&mut self, in_samples: &[u8], out_samples: &mut [f32]) {
        debug_assert!(in_samples.len() == out_samples.len());

        for i in 0..in_samples.len() {
            out_samples[i] = self.phase.sin();

            let omega = if in_samples[i] == 0 {
                self.omega_space
            } else {
                self.omega_mark
            };
            self.phase = (self.phase + self.sampling_period * omega).rem(2. * PI);
        }
    }
}
