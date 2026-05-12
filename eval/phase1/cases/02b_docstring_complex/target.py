def compute_moving_average(values, window, min_periods=None, pad_value=float('nan'), weights=None):
    if window <= 0:
        raise ValueError(f"window must be positive, got {window}")
    if min_periods is None:
        min_periods = window
    if not (0 <= min_periods <= window):
        raise ValueError(
            f"min_periods must be between 0 and {window}, got {min_periods}"
        )
    if weights is not None:
        if len(weights) != window:
            raise ValueError(
                f"weights length ({len(weights)}) must equal window ({window})"
            )
        if sum(weights) == 0:
            raise ValueError("weights must not sum to zero")

    if not values:
        return []

    result = []
    for i in range(len(values)):
        start = max(0, i - window + 1)
        chunk = values[start : i + 1]

        if len(chunk) < min_periods:
            result.append(pad_value)
        elif weights is None:
            result.append(sum(chunk) / len(chunk))
        else:
            w = weights[window - len(chunk) :]
            weighted_sum = sum(v * wt for v, wt in zip(chunk, w))
            result.append(weighted_sum / sum(w))

    return result
