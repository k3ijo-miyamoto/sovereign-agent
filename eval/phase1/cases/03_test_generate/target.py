def chunk(lst, size):
    if size <= 0:
        raise ValueError(f"size must be positive, got {size}")
    if not lst:
        return []
    return [lst[i:i + size] for i in range(0, len(lst), size)]
