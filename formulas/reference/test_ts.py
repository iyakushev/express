from timeseries.utils_window_functions import ema_calc
from timeseries.omen import emaf2, ema2
import numpy as np


def test_ema():
    value = np.array([2.0, 5.0, 1.0, 2.0], dtype=np.float64)
    time = np.array([0.0, 1.0, 3.0, 4.0], dtype=np.float64)
    window = 3
    result = ema_calc(value, time, window, np.float16(2.0))
    expected = [2, 3.6725, 2.4177, 2.3078]
    for pos, res in enumerate(result):
        assert res - expected[pos] < 1e-4


def test_ema_longer():
    value = np.array(
        [
            2.0,
            2.7,
            3.0,
            3.4,
            3.8,
            4.0,
            4.1,
            4.0,
            4.2,
            4.4,
            4.9,
            5.0,
            5.1,
            4.9,
        ],
        dtype=np.float64,
    )
    time = np.array(
        [
            0.0,
            1.0,
            1.3,
            1.6,
            1.9,
            2.0,
            2.1,
            2.15,
            2.3,
            2.6,
            2.8,
            3.1,
            3.2,
            3.3,
        ],
        dtype=np.float64,
    )
    window = 3
    result = ema_calc(value, time, window, np.float16(2.0))
    expected = [
        2.0,
        2.39026688,
        2.6183329,
        2.84202394,
        3.06688374,
        3.24761548,
        3.38862127,
        3.47626687,
        3.56979791,
        3.67080119,
        3.80972098,
        3.93593381,
        4.04937313,
        4.12562808,
    ]
    for pos, res in enumerate(result):
        assert res - expected[pos] < 1e-4


def test_omen_ema():
    value = np.array([2.0, 5.0, 1.0, 2.0], dtype=np.float64)
    time = np.array([0.0, 1.0, 3.0, 4.0], dtype=np.float64)
    window = 3
    result = ema2(value, time, window)
    resultf = emaf2(value, time, window)
    breakpoint()
    print(result)
    print(resultf)
