use crossbeam_channel::Sender;
use std::collections::VecDeque;

pub struct UartRx {

    // TODO: coloque outros atributos que você precisar aqui
    samples_per_symbol: usize,
    to_pty: Sender<u8>,
    curbyte: u8,
    pos_frame: usize,
    inside_frame: bool,
}

impl UartRx {
    pub fn new(samples_per_symbol: usize, to_pty: Sender<u8>) -> Self {
        // TODO: inicialize seus novos atributos abaixo
        UartRx {
            samples_per_symbol,
            to_pty,
            curbyte: 0,
            pos_frame: 0,
            inside_frame: false
        }
    }

    pub fn put_samples(&mut self, buffer: &[u8]) {
        // TODO: seu código aqui

        for sample in buffer {
            if !self.inside_frame {
                if *sample == 0 {
                    self.inside_frame = true;
                    self.pos_frame = 0;
                    self.curbyte = 0
                }
            } else {
                self.pos_frame += 1;
                if self.pos_frame % self.samples_per_symbol == self.samples_per_symbol / 2 {
                    self.curbyte = (self.curbyte >> 1) | (sample << 7);
                    if self.pos_frame / self.samples_per_symbol == 8 {

                        self.to_pty.send(self.curbyte).unwrap();

                    }

                    if self.pos_frame / self.samples_per_symbol == 9 {
                        self.inside_frame = false
                    }
                }
            }
        }
        //self.to_pty.send(65).unwrap();  // TODO: remova esta linha, é um exemplo de como mandar um byte para a pty
    }
}

pub struct UartTx {
    samples_per_symbol: usize,
    samples: VecDeque<u8>,
}

impl UartTx {
    pub fn new(samples_per_symbol: usize) -> Self {
        Self {
            samples_per_symbol,
            samples: VecDeque::new(),
        }
    }

    fn put_bit(&mut self, bit: u8) {
        for _ in 0..self.samples_per_symbol {
            self.samples.push_back(bit);
        }
    }

    pub fn put_byte(&mut self, mut byte: u8) {
        self.put_bit(0); // start bit
        for _ in 0..8 {
            self.put_bit(byte & 1);
            byte >>= 1;
        }
        self.put_bit(1); // stop bit
    }

    pub fn get_samples(&mut self, buffer: &mut [u8]) {
        for i in 0..buffer.len() {
            buffer[i] = self.samples.pop_front().unwrap_or(1);
        }
    }
}
