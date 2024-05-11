#include <avr/boot.h>

#define CHANNELS 4

float V[] = {10, 20, 30, 40};
float I[] = {1, 2, 3, 4};
bool active[CHANNELS] = {true, true, true, true};

void setup() {
  Serial.begin(9600);
}

void loop() {
  if (Serial.available()) {
    String i = Serial.readStringUntil('\n');
    if(i.startsWith("*IDN?")) {
      Serial.print("supply, 100MQ, ");
      for(int i = 0; i < 0xf; i++) {
        Serial.print(boot_signature_byte_get(i));
      }
      Serial.println(", x");
    } else if(i.startsWith("OPALL")) {
      for(int ch = 0; ch < 4; ch++) {
        active[ch] = i[6] == '1';
      }
    } else if(i[0] == 'V') {
      int n = (i[1] - '0') - 1;
      if(n < CHANNELS) {
        if(i[2] == 'O' && i[3] == '?') {
          float val = active[n] * (V[n] + random(-100, 100) / 100.0);
          Serial.print(val, 3);
          Serial.println("V");
        } else if(i[2] == '?') {
          Serial.print("V");
          Serial.print((char) (n + 1 + '0'));
          Serial.print(' ');
          Serial.println(V[n], 3);
        } else if(i[2] == ' ') {
          V[n] = atof(i.substring(3).c_str());
        }
      }
    } else if(i[0] == 'I') {
      int n = (i[1] - '0') - 1;
      if(n < CHANNELS) {
        if(i[2] == 'O' && i[3] == '?') {
          float val = active[n] * (I[n] + random(-100, 100) / 100.0);
          Serial.print(val, 3);
          Serial.println("I");
        } else if(i[2] == '?') {
          Serial.print("I");
          Serial.print((char) (n + 1 + '0'));
          Serial.print(' ');
          Serial.println(I[n], 3);
        } else if(i[2] == ' ') {
          I[n] = atof(i.substring(3).c_str());
        }
      }
    } else if(i[0] == 'O' && i[1] == 'P') {
      int n = (i[2] - '0') - 1;
      if(n < CHANNELS) {
        if(i[3] == '?') {
          Serial.println(active[n] ? '1' : '0');
        } else {
          active[n] = i[4] - '0';
        }
      }
    } else {
      Serial.println("Unknown");
    }
  }
}
