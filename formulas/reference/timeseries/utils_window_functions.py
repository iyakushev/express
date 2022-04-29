import numpy as np
import pandas as pd
from numba import njit
from .jurik_mov_avg import jma


def ma_calc(feature: np.array, time_ns: np.array, lookback_ns: int):
    len_feat = feature.shape[0]
    ma_feat = np.full(feature.shape, np.nan)

    cur_ma_val = 0
    cur_num_val = 0
    l_time_idx = len_feat - 1

    for r_time_idx in range(len_feat - 1, -1, -1):
        while (
            l_time_idx >= 0 and time_ns[r_time_idx] - time_ns[l_time_idx] <= lookback_ns
        ):

            cur_num_val += 1
            cur_ma_val = (
                cur_ma_val * (cur_num_val - 1) / cur_num_val
                + feature[l_time_idx] / cur_num_val
            )
            l_time_idx = l_time_idx - 1

        ma_feat[r_time_idx] = cur_ma_val
        cur_ma_val = (
            (cur_ma_val - feature[r_time_idx] / cur_num_val)
            * cur_num_val
            / np.maximum(cur_num_val - 1, 1)
        )
        cur_num_val = cur_num_val - 1

    return ma_feat


"""
This is actual MA function from qb
exp = 2 for qb's MA
"""


def ema_calc(
    feature: np.array, time_ns: np.array, lookback_ns: int, exp: np.float16 = 2
):
    len_feat = feature.shape[0]
    ma_feat = np.full(feature.shape, np.nan)

    cur_ma_val = 0
    l_time_idx = len_feat - 1

    for r_time_idx in range(len_feat - 1, -1, -1):
        l_time_idx = r_time_idx
        tdelta = 0
        tdelta_sum = 0
        exp_sum = 0
        exp_alpha = 1
        cur_ma_val = 0

        while tdelta_sum < lookback_ns:
            tdelta_sum += tdelta
            exp_alpha = np.exp(-np.log(exp) * tdelta_sum / lookback_ns)
            exp_sum += exp_alpha
            cur_ma_val += feature[l_time_idx] * exp_alpha

            l_time_idx = l_time_idx - 1

            if l_time_idx < 0:
                break

            tdelta = time_ns[l_time_idx + 1] - time_ns[l_time_idx]
            tdelta = np.minimum(lookback_ns - tdelta_sum, tdelta)

        ma_feat[r_time_idx] = cur_ma_val / exp_sum

    return ma_feat


"""
This is actual MALIN function from qb
"""


@njit
def malin_calc(feature: np.array, time_ns: np.array, lookback_ns: int):
    len_feat = feature.shape[0]
    ma_feat = np.full(feature.shape, np.nan)
    l_idx_prev = len_feat - 1

    for r_idx in range(len_feat - 1, -1, -1):
        for l_idx in range(r_idx, -2, -1):
            if time_ns[r_idx] - time_ns[l_idx] > lookback_ns:
                break

        linear_weights = np.arange(1, r_idx - l_idx + 1).astype(np.float64)
        ma_feat[r_idx] = (
            feature[l_idx + 1 : r_idx + 1].astype(np.float64) @ linear_weights
        )
        ma_feat[r_idx] /= np.sum(linear_weights)

    return ma_feat


@njit
def twa_feature_calc(feature: np.array, time_ns: np.array, lookback_ns: int):
    data_len = time_ns.shape[0]
    twa_feature = np.full(data_len, np.nan)
    r_idx_prev = 0

    for r_idx in range(data_len - 1, 0, -1):
        for l_idx in range(r_idx - 1, 0, -1):
            # for l_idx in range(data_len):
            #     for r_idx in range(r_idx_prev, data_len):
            if time_ns[r_idx] - time_ns[l_idx] > lookback_ns:
                break

        # int -> float cast for @ -multipliaction
        time_diff = np.diff(time_ns[l_idx : r_idx + 1]).astype(np.float64)

        twa_feature[r_idx] = (feature[l_idx + 1 : r_idx + 1] @ time_diff) / (
            np.sum(time_diff) + 0.0000001
        )
        # break

    return twa_feature


def ut_diff_twa(df, lb_ns: int, feature_name=None, time_col_name=None):
    if time_col_name is None:
        time_ns = np.array(df.index, dtype=np.int64)  # .reshape(1, -1).squeeze()
    else:
        time_ns = np.array(df.loc[:, time_col_name]).astype(
            np.int64
        )  # .reshape(1, -1).squeeze()

    if feature_name is None:
        feat_val = df.iloc[:, 0].values.astype(np.float64)  # .reshape(1, -1).squeeze()
        feature_name = df.columns[0]
    else:
        feat_val = df.loc[:, feature_name].values.astype(
            np.float64
        )  # .reshape(1, -1).squeeze()

    diff_twa = feat_val - twa_feature_calc(
        feature=feat_val, time_ns=time_ns, lookback_ns=lb_ns
    )

    return pd.DataFrame(
        diff_twa, index=time_ns, columns=[f"twa_diff_{feature_name}_{lb_ns}"]
    )


def ut_diff_malin(df, lb_ns: int, feature_name=None, time_col_name=None):
    if time_col_name is None:
        time_ns = np.array(df.index, dtype=np.int64)  # .reshape(1, -1).squeeze()
    else:
        time_ns = np.array(df.loc[:, time_col_name]).astype(
            np.float64
        )  # .reshape(1, -1).squeeze()

    if feature_name is None:
        feat_val = df.iloc[:, 0].values.astype(np.float64)  # .reshape(1, -1).squeeze()
        feature_name = df.columns[0]
    else:
        feat_val = df.loc[:, feature_name].values  # .reshape(1, -1).squeeze()

    diff_malin = feat_val - malin_calc(
        feature=feat_val, time_ns=time_ns, lookback_ns=lb_ns
    )

    return pd.DataFrame(
        diff_malin, index=time_ns, columns=[f"malin_diff_{feature_name}_{lb_ns}"]
    )


def ut_diff_ema(df, lb_ns: int, feature_name=None, time_col_name=None):

    if time_col_name is None:
        time_ns = np.array(df.index, dtype=np.int64)  # .reshape(1, -1).squeeze()
    else:
        time_ns = df.loc[:, time_col_name].values.astype(
            np.int64
        )  # .reshape(1, -1).squeeze()

    if feature_name is None:
        feat_val = df.iloc[:, 0].values.astype(np.float64)  # .reshape(1, -1).squeeze()
        feature_name = df.columns[0]
    else:
        feat_val = df.loc[:, feature_name].values.astype(
            np.float64
        )  # .reshape(1, -1).squeeze()
    diff_ema = feat_val - ema_calc(feature=feat_val, time_ns=time_ns, lookback_ns=lb_ns)

    return pd.DataFrame(
        diff_ema, index=time_ns, columns=[f"ema_diff_{feature_name}_{lb_ns}"]
    )


def ut_diff_jma(
    df, feature_name=None, time_col_name=None, length=None, phase=None, offset=None
):

    if time_col_name is None:
        time_ns = np.array(df.index, dtype=np.int64)  # .reshape(1, -1).squeeze()
    else:
        time_ns = df.loc[:, time_col_name].values.astype(
            np.int64
        )  # .reshape(1, -1).squeeze()

    if feature_name is None:
        feat_val = df.iloc[:, 0].values.astype(np.float64)  # .reshape(1, -1).squeeze()
        feature_name = df.columns[0]
    else:
        feat_val = df.loc[:, feature_name].values.astype(
            np.float64
        )  # .reshape(1, -1).squeeze()

    # length=None, phase=None, offset=None):

    diff_jma = feat_val - jma(feat_val, length=length, phase=phase, offset=offset)

    if length is None:
        length = 7

    if phase is None:
        phase = 0

    if offset is None:
        offset = 0

    return pd.DataFrame(
        diff_jma,
        index=time_ns,
        columns=[f"jma_diff_{feature_name}_{length}_{phase}_{offset}"],
    )


def ut_jma(
    df, feature_name=None, time_col_name=None, length=None, phase=None, offset=None
):

    if time_col_name is None:
        time_ns = np.array(df.index)  # .reshape(1, -1).squeeze()
    else:
        time_ns = df.loc[:, time_col_name].values.astype(
            np.float64
        )  # .reshape(1, -1).squeeze()

    if feature_name is None:
        feat_val = df.iloc[:, 0].values.astype(np.float64)  # .reshape(1, -1).squeeze()
        feature_name = df.columns[0]
    else:
        feat_val = df.loc[:, feature_name].values  # .reshape(1, -1).squeeze()

    # length=None, phase=None, offset=None):

    feat_jma = jma(feat_val, length=length, phase=phase, offset=offset)

    if length is None:
        length = 7

    if phase is None:
        phase = 0

    if offset is None:
        offset = 0

    return pd.DataFrame(
        feat_jma,
        index=time_ns,
        columns=[f"jma_{feature_name}_{length}_{phase}_{offset}"],
    )
