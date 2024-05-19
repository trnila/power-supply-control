#include <avr/boot.h>

#define CHANNELS 4

enum OnAction {
  Quick,
  Never,
  Delay,
};

struct Range {
   uint8_t voltage;
   float current;
};

float V[] = {10, 20, 30, 40};
float I[] = {1, 2, 3, 4};
bool active[CHANNELS] = {true, true, true, true};
OnAction onactions[CHANNELS] = {Quick, Quick, Quick, Quick};
uint32_t onaction_delays[CHANNELS];
int32_t channel_multi_on_at[CHANNELS] = {-1, -1, -1, -1};
Range range_limits[][4] = {
  {
    {.voltage = 0, .current = 0},
    {.voltage = 35, .current = 3},
    {.voltage = 16, .current = 6},
    {.voltage = 35, .current = 6},
  },
  {
    {.voltage = 0, .current = 0},
    {.voltage = 35, .current = 3},
    {.voltage = 70, .current = 1.5},
    {.voltage = 70, .current = 3},
  }
};
uint8_t ranges[] = {1, 1, 1, 1};

void setup() {
  Serial.begin(9600);
}

void loop() {
  for(int ch = 0; ch < CHANNELS; ch++) {
    if(onactions[ch] == Delay && channel_multi_on_at[ch] >= 0 && millis() >= channel_multi_on_at[ch]) {
      active[ch] = true;
      channel_multi_on_at[ch] = -1;
    }
  }

  if (Serial.available()) {
    String i = Serial.readStringUntil('\n');
    if(i.startsWith("*IDN?")) {
      Serial.print("supply, 100MQ, ");
      for(int i = 0; i < 0xf; i++) {
        Serial.print(boot_signature_byte_get(i));
      }
      Serial.println(", x");
    } else if(i.startsWith("OPALL")) {
      if(i[6] == '1') {
        for(int ch = 0; ch < CHANNELS; ch++) {
          if(onactions[ch] == Quick) {
            active[ch] = true;
          } else if(onactions[ch] == Delay) {
            channel_multi_on_at[ch] = millis() + onaction_delays[ch];
          }
        }
      } else {
        for(int ch = 0; ch < CHANNELS; ch++) {
          active[ch] = false;
        }
      }
    } else if(i.startsWith("VRANGE")) {
      int ch = (i[6] - '0') - 1;
      if(i[7] == '?') {
        Serial.println(ranges[ch]);
      } else {
        int vrange = i[8] - '0';
        if(vrange < 4) {
          ranges[ch] = vrange;
        }
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
          float req = atof(i.substring(3).c_str());
          if(req <= range_limits[n >> 1][ranges[n]].voltage) {
            V[n] = req;
          }
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
          float req = atof(i.substring(3).c_str());
          if(req <= range_limits[n >> 1][ranges[n]].current) {
            I[n] = req;
          }
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
    } else if(i.startsWith("ONACTION")) {
      int ch = (i[8] - '0') - 1;
      if(ch < CHANNELS) {
        String type = i.substring(10);
        channel_multi_on_at[ch] = -1;
        if(type == "QUICK") {
          onactions[ch] = Quick;
        } else if(type == "NEVER") {
          onactions[ch] = Never;
        } else if(type == "DELAY") {
          onactions[ch] = Delay;
        } else {
          Serial.println("INVALID ONACTION");
        }
      }
    } else if(i.startsWith("ONDELAY")) {
      int ch = (i[7] - '0') - 1;
      if(ch < CHANNELS) {
        onaction_delays[ch] = atoi(i.substring(9).c_str());
      }
    } else {
      Serial.println("Unknown");
    }
  }
}
