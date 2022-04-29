import numpy as np


def emaf2(price, time_, dt):
    """
    Builds future ema (on a whole series).

    Args:
        price(float64[:]): List of prices.
        time_(float64[:]/datetime in sec.): Corresponding time.
        dt(float): Period.

    Returns:
        List of EMA.
    """
    ema_pr = np.zeros(price.shape)
    ema_pr[-1] = price[-1]
    for n in range(len(time_) - 2, -1, -1):
        e = np.exp((time_[n] - time_[n + 1]) / dt * np.log(2))
        ema_pr[n] = e * ema_pr[n + 1] + (1 - e) * price[n + 1]
    return ema_pr


def ema2(time_, price, dt):
    """
    Builds ema (on a whole series).

    Args:
        price(float64[:]): List of prices.
        time_(float64[:]/datetime in sec.): Corresponding time.
        dt(float): Preiod.

    Returns:
        List of EMA.
    """
    ema_pr = np.zeros(price.shape)
    ema_pr[0] = price[0]
    for n in range(1, len(time_)):
        e = np.exp((time_[n - 1] - time_[n]) / dt * np.log(2))
        ema_pr[n] = e * ema_pr[n - 1] + (1 - e) * price[n - 1]
    return ema_pr
