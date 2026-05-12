def celsius_to_fahrenheit(c):
    return c * 9 / 5 + 32

temps = ["37", "100", "0"]
for t in temps:
    print(round(celsius_to_fahrenheit(t), 2))
