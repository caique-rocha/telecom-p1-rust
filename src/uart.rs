use crossbeam_channel::Sender;
use std::collections::VecDeque;

pub struct UartRx {

    samples_per_symbol: usize,
    to_pty: Sender<u8>,
    curbyte: u8,
    pos_frame: usize,
    inside_frame: bool,
}

impl UartRx {
    pub fn new(samples_per_symbol: usize, to_pty: Sender<u8>) -> Self {

        UartRx {
            samples_per_symbol,
            to_pty,
            curbyte: 0,
            pos_frame: 0,
            inside_frame: false
        }
    }

    pub fn put_samples(&mut self, buffer: &[u8]) {
        for &sample in buffer {
            if !self.inside_frame {
                // Procurando bit de start (0)
                if sample == 0 {
                    self.inside_frame = true;
                    self.pos_frame = 0;
                    self.curbyte = 0;
                }
            } else {
                self.pos_frame += 1;

                // Captura o bit no meio do símbolo
                if self.pos_frame % self.samples_per_symbol == self.samples_per_symbol / 2 {
                    // Desloca para a direita, armazena o bit mais significativo primeiro
                    self.curbyte >>= 1;
                    if sample == 1 {
                        self.curbyte |= 0b1000_0000;
                    }

                    // Se capturamos 8 bits (1 byte)
                    if self.pos_frame / self.samples_per_symbol == 8 {
                        // Enviar o byte construído para a PTY
                        self.to_pty.send(self.curbyte).unwrap();
                    }

                    // Se estamos no 9º bit, é o bit de stop, então resetamos o frame
                    if self.pos_frame / self.samples_per_symbol == 9 {
                        self.inside_frame = false;
                    }
                }
            }
        }
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
