import numpy as np
import pandas as pd
from tqdm.auto import tqdm
from typing import List, Dict, Tuple
from utils_window_functions import *
from models_features import Trade
# from custom_features import rsi_feature
from jurik_mov_avg import jma 

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# To prevent circular import from custom_features module
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

def rsi_feature(df, features, lookbacks):
    rsi_df = pd.DataFrame()
    for feature, lb in zip(features, lookbacks):
        feature_to_calc = pd.Series(df[feature].values, index=pd.to_datetime(df.index))
        up = np.maximum(np.sign(feature_to_calc), 0).rolling(window=f'{lb}N').sum()
        down = np.minimum(np.sign(feature_to_calc), 0).rolling(window=f'{lb}N').sum() 
        rsi_df[f'rsi_{feature}_{lb}'] =  up / (up + np.abs(down) + 1) 
    return rsi_df   

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

def SIG(feature: np.array, times_ns: np.array, ns_period: int) -> np.array:
    return feature - ema_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period)

def SIGLIN(feature: np.array, times_ns: np.array, ns_period: int) -> np.array:
    return feature - malin_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period)

def SIGT(feature: np.array, times_ns: np.array, ns_period: int) -> np.array:
    return feature - twa_feature_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period)

def SIGSFL(feature: np.array, times_ns: np.array, ns_period: int, gamma: float) -> np.array:
    return feature - sfl3(feature=feature, times_ns=times_ns, ns_period=ns_period, gamma=gamma)

def WSFLDELTA(feature: np.array, times_ns: np.array, ns_period_sfl: int, gamma: float, ns_period_wdelta: int) -> np.array:
    return wdelta(feature = sfl3(feature=feature, 
                                 times_ns=times_ns, 
                                 ns_period=ns_period_sfl, 
                                 gamma=gamma),
                  times_ns = times_ns,
                  ns_period=ns_period_wdelta)


def TWA(feature: np.array, times_ns: np.array, ns_period: int) -> np.array:
    return twa_feature_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period)

def EMA(feature: np.array, times_ns: np.array, ns_period: int) -> np.array:
    return ema_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period)

def EMA_DIFF(feature: np.array, times_ns: np.array, ns_period_1: int, ns_period_2: int) -> np.array:
    ema1 = ema_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period_1)
    ema2 = ema_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period_2)
    return ema1 - ema2

def TWA_DIFF(feature: np.array, times_ns: np.array, ns_period_1: int, ns_period_2: int) -> np.array:
    twa1 = twa_feature_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period_1)
    twa2 = twa_feature_calc(feature=feature, time_ns=times_ns, lookback_ns=ns_period_2)
    return twa1 - twa2

def JRSX(feature: np.array, times_ns: np.array, ns_period: int, length=None, phase=None, offset=None) -> np.array:
    
    to_rsi = pd.DataFrame(feature, index=times_ns, columns=['to_rsi'])
    rsi = rsi_feature(to_rsi, ['to_rsi'], [ns_period])
    return ut_jma(rsi, length=length, phase=phase, offset=offset).values.reshape(1, -1).squeeze()
    
def JVEL(feature: np.array, times_ns: np.array, ns_period: int, length=None, phase=None, offset=None) -> np.array:
    wind_returns = wret(feature, times_ns, ns_period)
    return jma(wind_returns, length=length, phase=phase, offset=offset)


def change(feature):
    return np.diff(feature)

@njit 
def last(feature, times_ns, ns_period: int):
    ret_vals = np.full(times_ns.shape, np.nan).astype(np.float64)
    for time_ind in range(len(times_ns)-1, -1, -1):
        cur_val = 0
        cur_ind = time_ind
        while cur_ind and times_ns[time_ind] - times_ns[cur_ind] < ns_period:
            if not np.isnan(feature[cur_ind]):
                ret_vals[time_ind] = feature[cur_ind]
                return
            cur_ind -= 1
            
    return ret_vals

@njit 
def wmin(feature, times_ns, ns_period: int):
    ret_vals = np.full(times_ns.shape, np.inf).astype(np.float64)
    for time_ind in range(len(times_ns)-1, -1, -1):
        cur_val = 0
        cur_ind = time_ind
        
        while cur_ind and times_ns[time_ind] - times_ns[cur_ind] < ns_period:
            ret_vals[time_ind] = np.minimum(feature[cur_ind], ret_vals[time_ind])
            cur_ind -= 1
    return ret_vals

@njit 
def wmax(feature, times_ns, ns_period: int):
    ret_vals = np.full(times_ns.shape, -np.inf).astype(np.float64)
    for time_ind in range(len(times_ns)-1, -1, -1):
        cur_val = 0
        cur_ind = time_ind
        while cur_ind and times_ns[time_ind] - times_ns[cur_ind] < ns_period:
            ret_vals[time_ind] = np.maximum(feature[cur_ind], ret_vals[time_ind])
            cur_ind -= 1    
    return ret_vals

@njit 
def eboxsum(feature, times_ns, ns_period: int):
    ret_vals = np.full(times_ns.shape, 0).astype(np.float64)
    for time_ind in range(len(times_ns)-1, -1, -1):
        cur_val = 0
        cur_ind = time_ind
        
        while cur_ind and times_ns[time_ind] - times_ns[cur_ind] < ns_period:
            ret_vals[time_ind] += feature[cur_ind]
            cur_ind -= 1
            
    return ret_vals

@njit
def wdelta(feature, times_ns, ns_period: int):
    delta_vals = np.full(times_ns.shape, 0).astype(np.float64)
    for time_ind in range(len(times_ns)-1, -1, -1):
        cur_val = 0
        cur_ind = time_ind
        
        while cur_ind and times_ns[time_ind] - times_ns[cur_ind] < ns_period:
            cur_ind -= 1
            
        delta_vals[time_ind] = feature[time_ind] - feature[cur_ind]
        
    return delta_vals

@njit
def wret(feature, times_ns, ns_period: int):
    ret_vals = np.full(times_ns.shape, 0).astype(np.float64)
    
        
    for time_ind in range(len(times_ns)-1, -1, -1):
        cur_val = 0
        cur_ind = time_ind
        
        while cur_ind and times_ns[time_ind] - times_ns[cur_ind] < ns_period:
            cur_ind -= 1
            
        ret_vals[time_ind] = (feature[time_ind] - feature[cur_ind]) / (feature[cur_ind] + 1e-7)
        
    return ret_vals

@njit 
def sfl3(feature, times_ns, ns_period: int, gamma: float):
    sfl_vals = np.full(times_ns.shape, 0).astype(np.float64)
    l1, l2, l3, l4 = 0, 0, 0, 0
    for time_ind in range(len(times_ns)):
    # for time_ind in range(len(times_ns)-1, -1, -1):
        cur_val = 0
        cur_ind = time_ind
        mmax = -1e9
        mmin = 1e9
        
        while cur_ind and times_ns[time_ind] - times_ns[cur_ind] < ns_period:
            mmax = max(mmax, feature[cur_ind])
            mmin = min(mmin, feature[cur_ind])
            cur_ind -= 1
            
        mid_candle = (mmax + mmin) / 2
        
        l1_ = (1 - gamma) * mid_candle + gamma * l1
        l2_ = -gamma * l1_ + l1 + gamma * l2
        l3_ = -gamma * l2_ + l2 + gamma * l3
        l4_ = -gamma * l3_ + l3 + gamma * l4
        
        l1, l2, l3, l4 = l1_, l2_, l3_, l4_
        
        sfl_vals[time_ind] = (l1 + 2 * l2 + 2 * l3 + l4) / 6
        # sfl_vals[time_ind] = (feature[time_ind] - feature[cur_ind]) / (feature[cur_ind] + 1e-7)
    return sfl_vals

# Presented in custom_features.py

def diffn():
    pass

# Presented in custom_features.py

def rolln():
    pass

# Presented in custom_features.py

def bookdepth():
    pass

# Presented in custom_features.py

def log3():
    pass

"""
This function is used to unify working with self-defined feature functions and pandas
:params:
feature_param_map: Dict -- should be in following format:
{ feature_name: [feature_params_dict] }

i.e.
{'wdelta': [{'ns_period': 60 * NANOSECOND}, {'ns_period': 60 * NANOSECOND}] } 


feature_df_col_name -- name of feature col in result dataframe must be presented

If feature_col_name is not specified in feature_params_dict .iloc[:, 0] got

If time_col_name is not specified in feature_params_dict index got
"""
def feature_scheduler(df: pd.DataFrame, feature_param_map: Dict[str, Dict], verbose: bool = False) -> List:
    if verbose:
        print(f"Calculate feature config with features")
        
    global_name_space = globals()
    res_feature = []
    for feature_name, feature_params_lst in feature_param_map.items():
        for feature_params in feature_params_lst:
            if feature_name not in global_name_space:
                print(f'Feature {feature_name} is not implemented')
                continue

            if 'feature_df_col_name' not in feature_params:
                print(f'For {feature_name} feature there are no df col name')
                continue
                
            if verbose:
                print(f"Start {feature_name} calculation, with parameters: {feature_params}")
            
            feature_df_col_name = feature_params['feature_df_col_name'] 

            feature_func = global_name_space[feature_name]

            if not feature_params.get('feature_col_name'):
                if isinstance(df, pd.Series):
                    values = df.values
                else:
                    values = df.iloc[:, 0].values.reshape(1, -1).squeeze()
            else:
                values = df[feature_params['feature_col_name']].values.reshape(1, -1).squeeze()

            if 'time_col_name' not in feature_params:
                times_ns = df.index.values.reshape(1, -1).squeeze()
            else:
                times_ns = df[feature_params['time_col_name']].values.reshape(1, -1).squeeze()

            additional_args = {arg_name: arg_val for arg_name, arg_val in feature_params .items()
                               if arg_name not in ('feature_col_name', 'time_col_name', 'feature_df_col_name')}

            feature_ar = feature_func(feature=values, times_ns=times_ns, **additional_args)
            res_feature.append(pd.DataFrame(feature_ar, index=times_ns, columns=[feature_df_col_name]))
        
    return res_feature
        