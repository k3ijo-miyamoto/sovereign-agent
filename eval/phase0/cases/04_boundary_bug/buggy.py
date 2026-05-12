def nth_fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return b

for i in [1, 2, 3, 4, 5]:
    print(nth_fibonacci(i))
