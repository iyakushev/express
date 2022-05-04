import numpy as np
import pandas as pd
import numba as nb

from numba import njit
from tqdm import tqdm
from typing import Callable, List, Tuple, Dict

NANOSECOND = 1
MICROSECOND = 1000 * NANOSECOND
MILLISECOND = 1000 * MICROSECOND
SECOND = 1000 * MILLISECOND

"""
B - count bid (buy)
String
trades only
A - count ask (sell)
trades only
C - return number of
deals instead of total
volume
Q - return total volume
in quote currency
G - use gaussian
convolution instead of
box sum
(Window is sigma then)
"""
def ANNVOL(trade_prcs, 
           trade_vols, 
           trade_times_ns, 
           times_on_ns, 
           flags: List[str], 
           window_ns: int,
           spread_vals: np.array):
    # numpy zip 
    trades = np.c_[trade_prcs, trade_vols]
    assert trade_times_ns.shape[0] == trades.shape[0], 'Something wrong with trades data'
    
    get_used_quantity = lambda t: t[1]
    trade_filter = lambda t: True

    if 'Q' in flags:
        get_used_quantity = lambda t: t[1] * t[0]
        
    elif 'C' in flags:
        get_used_quantity = lambda t: 1
    
    if 'A' in flags:
        trade_filter = lambda t: t[1] < 0
    
    if 'B' in flags:
        trade_filter = lambda t: t[1] > 0
        
    # trade_filter = nb.njit(trade_filter)
    
    # if 'O' in flags:
    #     trade_filter = lambda t, *s: trade_filter(t) and 
        
    get_used_quantity = nb.njit(get_used_quantity)
    trade_filter = nb.njit(trade_filter)
    times_ns_ = np.array(times_on_ns, dtype=np.int64) 
    
    @njit
    def _annvol(trades, trade_times_ns, times_ns, window_ns, spread):
        ret_vals = np.full(times_ns.shape, np.nan).astype(np.float64)
        window_trades = np.zeros((1_000_000, 2))

        cur_trade_ind = len(trade_times_ns) - 1
        start_ind_in = cur_trade_ind
        
        for time_ind in range(len(times_ns)-1, -1, -1):
            cur_trade_ind = start_ind_in

            while cur_trade_ind and times_ns[time_ind] - trade_times_ns[cur_trade_ind] < 0:
                cur_trade_ind -= 1
            
            start_ind_in = cur_trade_ind
            
            cur_window_last_ind = 0

            while cur_trade_ind and times_ns[time_ind] - trade_times_ns[cur_trade_ind] < window_ns:
                window_trades[cur_window_last_ind] = trades[cur_trade_ind]
                cur_window_last_ind += 1
                cur_trade_ind -= 1
                
            # If there are no trades within window
            if cur_trade_ind == start_ind_in:
                continue
            # print('asd')
            # input()
            cur_val = 0
            for t in window_trades[:cur_window_last_ind]:
                if spread_vals.shape[0]:
                    up = spread_vals[time_ind][1]
                    dn = spread_vals[time_ind][0]
                    if up > t[0] > dn:
                        continue
                        
                if trade_filter(t):
                    cur_val += get_used_quantity(t) 
                    
            ret_vals[time_ind] = cur_val
        return ret_vals
    if 'O' in flags:
        return _annvol(trades, trade_times_ns, times_ns_, window_ns, spread_vals)
    
    return _annvol(trades, trade_times_ns, times_ns_, window_ns, np.array([]))



@njit
def mean_sqrt_ask(values, times):
    m, l = 0, len(values)
    for val in values:
        if val < 0:
            m += np.sqrt(-val)
    return m / max(l, 1)

@njit
def mean_sqrt_bid(values, times):
    m, l = 0, len(values)
    for val in values:
        if val > 0:
            m += np.sqrt(val)
    return m / max(l, 1)


@njit
def mean_ask(values, times):
    m, l = 0, len(values)
    for val in values:
        if val < 0:
            m += -val
    return m / max(l, 1)

@njit
def mean_bid(values, times):
    m, l = 0, len(values)
    for val in values:
        if val > 0:
            m += val
    return m / max(l, 1)

@njit
def imbalance(values, times):
    m, norm = 0, 0
    for val in values:
        m += val
        norm += abs(val)
    return m / max(norm, 1)

@njit
def raw_imbalance(values, times):
    m, norm = 0, 0
    for val in values:
        m += np.sign(val)
        norm += 1
    return m / max(norm, 1)

@njit
def features_njit_wrapper(values, times, start_time, _):
    return  mean_sqrt_ask(values, times), mean_sqrt_bid(values, times), mean_ask(values, times), mean_bid(values, times), imbalance(values, times), raw_imbalance(values, times)

"""
times  in increasing order
values in decreasing order

start_time -- it is end_time actually (later than all times)
"""
@njit
def pressure(values, times, start_time, args = (False, np.nan)):
    use_count, time_factor_ns = args
    signed_vols = values
    # print(time_factor_ns)

    # print('wask')

    # make times in decreasing order
    # and adjust by start time
    def process_time(t, st):
        t_ = np.zeros(t.shape[0] + 1)
        t_[1:] = t[::-1]
        t_[0] = st
        dt_ = np.abs(np.diff(t_))
        return dt_
    
    times_ask = times[signed_vols < 0]
    dtimes_ask = process_time(times_ask, start_time)
    # print('wask')

    times_bid = times[signed_vols > 0]
    dtimes_bid = process_time(times_bid, start_time)
    # print('wask')

    if use_count: 
        asks = np.ones(signed_vols[signed_vols < 0].shape)
        bids = np.ones(signed_vols[signed_vols > 0].shape)
    else:
        asks = np.abs(signed_vols[signed_vols < 0])
        bids = signed_vols[signed_vols > 0]
        
    # print('shapes')    
    # print(asks.shape)
    # print(dtimes_ask.shape)
    
    def get_weights(v, dt):
        # w = np.zeros(dt.shape[0]+1)
        w = np.cumsum(dt)
        w_sum = np.sum(w)
        w = np.exp(-w /  w_sum)
        # if np.isnan(w)
        return w
    
    w_ask = get_weights(asks, dtimes_ask)
    w_bid = get_weights(bids, dtimes_bid)
    
    # print('wask')
    # print(asks)
    # print(w_ask)
    # print('wbid')
    # print(bids)
    # print(w_bid)
    # print(f'tf: {time_factor_ns}')
    # input()
    if np.isnan(time_factor_ns):
        # print(f"pr returns: {bids @ w_bid - asks @ w_ask}")
        return bids @ w_bid - asks @ w_ask
    
    bid_time_factor = np.exp(-np.log(2) * dtimes_bid[0] / time_factor_ns)
    ask_time_factor = np.exp(-np.log(2) * dtimes_bid[0] / time_factor_ns)
    
    return bid_time_factor * bids @ w_bid  - asks @ w_ask * ask_time_factor


    
trade_features_names = ['sqrt_vol_mean_ask', 'sqrt_vol_mean_bid', 'vol_mean_ask', 'vol_mean_bid', 'imbalance', 'raw_imbalance']


"""
todo(akarpov)
Fun. description 
"""
@njit
def calc_window_functions(time_on, time_in, values_in, Ts_nano: np.array, njit_funcs_wrapper, num_njit_funcs, funcs_args):
    # print(111)
    features_arrs = np.zeros((len(Ts_nano), num_njit_funcs, len(time_on)))
    # print(111)
    for T_ind, (T_nano, func_args) in enumerate(zip(Ts_nano, funcs_args)):       
        cur_ind_on, cur_ind_in = len(time_on) - 1, len(time_in) - 1        
        window_vals = np.zeros(1_000_000)
        start_ind_in = cur_ind_in
        
        for cur_ind_on in range(len(time_on)-1, -1, -1):
            cur_ind_in = start_ind_in
            
            while cur_ind_in and time_on[cur_ind_on] - time_in[cur_ind_in] < 0:
                cur_ind_in -= 1
                
            start_ind_in = cur_ind_in 
            cur_window_last_ind = 0

            while cur_ind_in and time_on[cur_ind_on] - time_in[cur_ind_in] < T_nano:
                window_vals[cur_window_last_ind] = values_in[cur_ind_in]
                cur_window_last_ind += 1
                cur_ind_in -= 1
            
            # If there are no time_in events
            if cur_ind_in == start_ind_in:
                continue

            features_arrs[T_ind, :, cur_ind_on] = njit_funcs_wrapper(args=func_args,
                                                                     values = window_vals[:cur_window_last_ind], 
                                                                     times = time_in[cur_ind_in + 1: start_ind_in + 1],
                                                                     start_time=time_on[cur_ind_on])
            
    # time normalization
    for T_ind, T_nano in enumerate(Ts_nano):
        T_sec = T_nano / SECOND
        features_arrs[T_ind, :] /= T_sec
        
    return features_arrs


def get_trades_nprepr(df_trades):
    times = df_trades.index.values
    signed_volumes = ((2 * (df_trades['aggro_side'] == 'BID') - 1) * df_trades['size']).values
    prices = df_trades['price']
    return times, signed_volumes, prices


def ANNVOL_df(df_trades, 
              times_on_ns, 
              flags: List[str], 
              window_ns: int, 
              feat_pref: str = '',
              spread_vals = np.array([])):
    trd_ts, sig_vol, trd_prc = get_trades_nprepr(df_trades)
    anvol_df_list = [0] * len(flags)
    
    for ind, fl in enumerate(flags):
        annvol_vals = ANNVOL(trd_prc, sig_vol, trd_ts, times_on_ns, fl, window_ns, spread_vals)
        anvol_df_list[ind] = pd.DataFrame(annvol_vals, index=times_on_ns, columns=[f'{feat_pref}_annvol_{window_ns}_{fl}'])
    
    return anvol_df_list
    