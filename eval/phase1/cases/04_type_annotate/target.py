def parse_csv_row(line, delimiter, strip_whitespace):
    fields = line.split(delimiter)
    if strip_whitespace:
        fields = [f.strip() for f in fields]
    return fields


def count_words(text, ignore_case):
    if ignore_case:
        text = text.lower()
    words = text.split()
    counts = {}
    for word in words:
        counts[word] = counts.get(word, 0) + 1
    return counts


def clamp(value, min_value, max_value):
    return max(min_value, min(max_value, value))
