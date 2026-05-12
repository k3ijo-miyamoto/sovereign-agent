def find_max(numbers):
    max_val = numbers[0]
    for n in numbers:
        if n < max_val:
            max_val = n
    return max_val

print(find_max([3, 1, 9, 4, 7]))
print(find_max([20, 5, 11, 2]))
