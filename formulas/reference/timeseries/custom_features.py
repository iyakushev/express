# import features
import numpy as np
import pandas as pd

from copy import deepcopy
from collections import defaultdict
from constants import intrument_lotsize
from itertools import product
from numba import njit
from typing import List, Dict, Tuple, Optional
from features import eboxsum
from trade_features import ANNVOL_df
from features import feature_scheduler

# In pipeline time in nanoseconds

NANOSECOND = 1
MICROSECOND = 1000 * NANOSECOND
MILLISECOND = 1000 * MICROSECOND
SECOND = 1000 * MILLISECOND

num_of_levels = 20




# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ lob functions 

def vwap_all_by_size(df, up_size, instrument):
    vwap_side = pd.DataFrame()
    sum_vol = defaultdict(int)
    vwap = 0
    
    for side in ('ask', 'bid'):
        for lvl in range(num_of_levels):
            cur_vol_mult = pd.max(pd.min(up_size - sum_vol, df[f'{instrument}_{side}_vol_{lvl}']), 0)
            sum_vol[side] += cur_vol_mult
            vwap += cur_vol_mult * df[f'{instrument}_{side}_price_{lvl}']
            # prev_rem_vol = 
        
    vwap_side[f'{instrument}_vwap_all_qty_{str(up_size).replace(".","p")}'] = vwap / sum_vol
    
    return vwap_side


def vwap_by_size(df, up_size, side, instrument):
    vwap_side = pd.DataFrame()
    sum_vol = 0
    vwap = 0
    prev_rem_vol = 0
    assert side in ('ask', 'bid'), f'There are no such side: {side}'
    for lvl in range(num_of_levels):
        cur_vol_mult = pd.max(pd.min(up_size - sum_vol, df[f'{instrument}_{side}_vol_{lvl}']), 0)
        sum_vol += cur_vol_mult
        vwap += cur_vol_mult * df[f'{instrument}_{side}_price_{lvl}']
        # prev_rem_vol = 
        
    vwap_side[f'{instrument}_vwap_{side}_qty_{str(up_size).replace(".","p")}'] = vwap / sum_vol
    
    return vwap_side

def vmid(df, up_level, side, instrument):
    '''
    volume-weighted midprice calculated on levels 
    up to up_level param, on specific side 
    '''
        
    vmid = pd.DataFrame()
    assert side in ('ask', 'bid'), f'There are no such side: {side}'
    sum_vol = sum([df[f'{instrument}_{side}_vol_{lvl}'] for lvl in range(up_level+1)])
    vwap = sum([df[f'{instrument}_{side}_vol_{lvl}'] * df[f'{instrument}_{side}_price_{lvl}'] for lvl in range(up_level+1)])
    
    vmid[f'{instrument}_{side}_vmid_{up_level}'] = vwap / sum_vol
    return vmid

def midprices_lvl_calc(df, level, instruments):
    df_midprices = pd.DataFrame()
    
    for inst in instruments:
        df_midprices[f'{inst}_midprice_{level}'] = (df[f'{inst}_ask_price_{level}'] + df[f'{inst}_bid_price_{level}']) / 2
    return df_midprices
    
def diff_feature(features, lookbacks):
    '''
    calculate diff of feature and returns pd.DataFrame 
    with proper naming 
    '''
    diffs = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            feat_name = f'diff_{lb}_{feature}'
            diffs[feat_name] = features[feature].diff(periods=lb)
    return diffs


def vwap_all(df, up_level, instrument):
    '''
    volume-weighted midprice calculated on levels 
    up to up_level param, on specific side 
    ''' 
    vwap_all = pd.DataFrame()

    sum_vol = sum([df[f'{instrument}_ask_vol_{lvl}'] for lvl in range(up_level+1)])
    sum_vol += sum([df[f'{instrument}_bid_vol_{lvl}'] for lvl in range(up_level+1)])
    
    vwap = sum([df[f'{instrument}_ask_vol_{lvl}'] * df[f'{instrument}_ask_price_{lvl}'] for lvl in range(up_level+1)])
    vwap += sum([df[f'{instrument}_bid_vol_{lvl}'] * df[f'{instrument}_bid_price_{lvl}'] for lvl in range(up_level+1)])

    vwap_all[f'{instrument}_vwap_all_{up_level}'] = vwap / sum_vol
    return vwap_all


def vwap_side(df, up_level, side, instrument=''):
    '''
    volume-weighted midprice calculated on levels 
    up to up_level param, on specific side 
    ''' 
    vwap_side = pd.DataFrame()
    assert side in ['ask', 'bid'], f'There are no such side: {side}'
    sum_vol = sum([df[f'{instrument}_{side}_vol_{lvl}'] for lvl in range(up_level+1)])
    vwap = sum([df[f'{instrument}_{side}_vol_{lvl}'] * df[f'{instrument}_{side}_price_{lvl}'] for lvl in range(up_level+1)])
    
    vwap_side[f'{instrument}_vwap_{side}_{up_level}'] = vwap / sum_vol
    return vwap_side

def midprices_lvl_calc(df, level, instruments):
    df_midprices = pd.DataFrame()
    
    for inst in instruments:
        df_midprices[f'{inst}_midprice_{level}'] = (df[f'{inst}_ask_price_{level}'] + df[f'{inst}_bid_price_{level}']) / 2
    return df_midprices

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ features

def ma_diffs_calc(df, features_1, features_2=None, periods: List[Tuple[int, int]]= None):
    if features_2 is None:
        features_2 = features_1
        
    ma_diff_features_df = pd.DataFrame()
    for per_1, per_2 in periods:
        for feat_1_name, feat_2_name in zip(features_1, features_2):
            ma_diff_features_df[f'ma_{feat_1_name}_{per_1}_diff_{feat_2_name}_{per_2}'] = (df[feat_1_name].rolling(min_periods=1, window=per_1).mean() - df[feat_2_name].rolling(min_periods=1, window=per_2).mean())
    return ma_diff_features_df

def squared_diffs_calc(df, features, lookbacks):
    df_volatility = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            df_volatility[f'volat_{feature}_{lb}'] = ((df[feature] - df[feature].shift(periods=1))**2).rolling(min_periods=1, window=lb).mean()
    return df_volatility

def squared_diffs_semi_calc(df, features, lookbacks):
    df_volatility = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            diff = (df[feature] - df[feature].shift(periods=1))
            df_volatility[f'semi_p_volat_{feature}_{lb}'] = ((diff * diff * (diff > 0))**2).rolling(min_periods=1, window=lb).mean()
            df_volatility[f'semi_n_volat_{feature}_{lb}'] = ((diff * diff * (diff < 0))**2).rolling(min_periods=1, window=lb).mean()
            
    return df_volatility

"""
Or quadratic variation
"""
def realized_volatility(df, features, lookbacks):
    df_volatility = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            log_diff_sq = (np.log(df[feature]) - np.log(df[feature].shift(periods=1)))**2
            df_volatility[f'real_volat_{feature}_{lb}'] = (log_diff_sq).rolling(min_periods=1, window=lb).mean()
            
    return df_volatility

def bipower_variation(df, features, lookbacks):
    df_volatility = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            log_ret = (np.log(df[feature]) - np.log(df[feature].shift(periods=1)))
            df_volatility[f'bivar_{feature}_{lb}'] = (np.abs(log_ret) * np.abs(log_ret.shift(1))).rolling(min_periods=1, window=lb).mean()
            
    return df_volatility

def noise_variance(df, features, lookbacks):
    df_volatility = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            log_diff_sq = (np.log(df[feature]) - np.log(df[feature].shift(periods=1)))
            df_volatility[f'noise_var_{feature}_{lb}'] = -(log_diff_sq * log_diff_sq.shift(1)).rolling(window=lb).sum() * np.pi / 2 # for scaling reason
    return df_volatility


def jump_variation(real_vols_df, bi_vols_df):
    df_jmp_var = pd.DataFrame()
    for ind, col in enumerate(real_vols_df.columns):
        df_jmp_var[f'jump_var{col.split("volat")[-1]}'] = np.maximum(real_vols_df[col] - bi_vols_df.iloc[:, ind], 0)
        
    return df_jmp_var
            

def realized_quarticity(df, features, lookbacks, time_to_norm=None):
    df_quarticity = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            quarticity = (df[feature] - df[feature].shift(periods=1))**4
            if time_to_norm is not None:
                quarticity = pd.Series((quarticity.values / time_to_norm).squeeze())
                
            df_quarticity[f'real_quart_{feature}_{lb}'] = (quarticity).rolling(window=lb).mean()

    df_quarticity.index = df.index
    return df_quarticity


def side_compactness_fraq(df_lob, instruments, instrument_minstep: Dict, up_level=20, sides = ('bid', 'ask')):
    assert all([inst in instrument_minstep for inst in instruments]), 'All instrument minsteps should be presented'
    
    side_compactness = pd.DataFrame()
    for inst in instruments:
        for side in sides:
            assert side in ('bid', 'ask')
            side_sign = 2 * (side == 'ask') - 1

            side_compactness[f'{inst}_{side}_comp_fraq_{up_level}'] = (df_lob[f'{inst}_{side}_price_{up_level-1}'] - df_lob[f'{inst}_{side}_price_{0}']) * side_sign / instrument_minstep[inst] + 1 - up_level
             
    return side_compactness
        
        
        
def side_density_mean(df_lob, instruments, up_level=20, sides = ('bid', 'ask')):
    side_mean = pd.DataFrame()
    
    for inst in instruments:
        for side in sides:
            assert side in ['bid', 'ask']
            side_sign = 2 * (side == 'ask') - 1
            best_side = df_lob[f'{inst}_{side}_vol_{0}']
            sum_vol = sum([df_lob[f'{inst}_{side}_vol_{lvl}'] for lvl in range(up_level)]) + 1

            side_mean[f'{inst}_{side}_density_mean'] = sum([(df_lob[f'{inst}_{side}_price_{lvl}'] - df_lob[f'{inst}_{side}_price_{0}']) * df_lob[f'{inst}_{side}_vol_{lvl}'] / sum_vol * side_sign
                                            for lvl in range(up_level)])
    return side_mean
        

def side_density_variance(df_lob, instruments, up_level=20, sides = ('bid', 'ask'), side_means = None):
    side_var = pd.DataFrame()
    if side_means is not None:
        side_mean = side_means
    else:
        side_mean = side_density_mean(df_lob, instruments=instruments, up_level=up_level, sides=sides)
        
    for inst in instruments:
        for side in sides:
            assert side in ['bid', 'ask']
            side_sign = 2 * (side == 'ask') - 1
            best_side = df_lob[f'{inst}_{side}_vol_{0}']
            sum_vol = sum([df_lob[f'{inst}_{side}_vol_{lvl}'] for lvl in range(up_level)]) + 1

            side_var[f'{inst}_{side}_density_var'] = sum([(df_lob[f'{inst}_{side}_price_{lvl}'] - df_lob[f'{inst}_{side}_price_{0}'])**2 * df_lob[f'{inst}_{side}_vol_{lvl}'] / sum_vol * side_sign
                                            for lvl in range(up_level)])
            side_var[f'{inst}_{side}_density_var'] -= side_mean[f'{inst}_{side}_density_mean']
            
            side_var[f'{inst}_{side}_density_var'].clip(None, 100, inplace=True) 
            
    return side_var


def diff_level_prc_corr(df, up_level, inst):
    assert up_level < 20 - 1
    diff_lvl_corr = pd.DataFrame()
    diff_lvl_corr[f'{inst}_diff_lvl_corr'] = sum([(df[f'{inst}_ask_price_{lvl+1}'] - df[f'{inst}_ask_price_{lvl}']) * 
                                                  (df[f'{inst}_bid_price_{lvl}'] - df[f'{inst}_bid_price_{lvl+1}']) 
                                                  for lvl in range(up_level)]) / up_level
    return diff_lvl_corr


def vimba_level(df, levels, instruments):
    features = product(levels, instruments)
    vimba_at_level = pd.DataFrame()
    for lvl, inst in features:
        vimba_at_level[f'{inst}_vimba_at_{lvl}'] = (df[f'{inst}_ask_vol_{lvl}'])/ (df[f'{inst}_ask_vol_{lvl}'] + df[f'{inst}_bid_vol_{lvl}'])
    return vimba_at_level


def vimba_up_level(df, up_level, instruments):
    vimba_up_level = pd.DataFrame()
    for inst in instruments:
        ask = sum([df[f'{inst}_ask_vol_{lvl}'] for lvl in range(up_level)])
        vimba_up_level[f'{inst}_vimba_up_{up_level}'] = ask / (sum([df[f'{inst}_bid_vol_{lvl}'] for lvl in range(up_level)]) + ask)
    return vimba_up_level

def return_price_across_level(df, levels: List[int], instruments: List[str], sides=('ask','bid')):
    global num_of_levels
    assert all([lvl < num_of_levels-2 for lvl in levels])
    
    features = product(levels, instruments, sides)
    price_return_level = pd.DataFrame()
    for lvl, inst, side in features:
        price_return_level[f'{inst}_price_return_level_{lvl}_{side}'] = (df[f'{inst}_{side}_price_{lvl}'] - df[f'{inst}_{side}_price_{lvl+1}'])/ df[f'{inst}_{side}_price_{lvl+1}']
    return price_return_level

def vmid_diff_mid(df, instrument, up_level):
    vmid_diff_mid_df = pd.DataFrame()
    vmid_diff_mid_df[f'{instrument}_vmid_diff_mid_{up_level}'] = vwap_all(df, instrument=instrument, up_level=up_level).iloc[:,0] - df[f'{instrument}_midprice_0']
    return vmid_diff_mid_df

def rsi_feature(df, features, lookbacks):
    rsi_df = pd.DataFrame()
    for feature, lb in zip(features, lookbacks):
        feature_to_calc = pd.Series(df[feature].values, index=pd.to_datetime(df.index))
        up = np.maximum(np.sign(feature_to_calc), 0).rolling(window=f'{lb}N').sum()
        down = np.minimum(np.sign(feature_to_calc), 0).rolling(window=f'{lb}N').sum() 
        rsi_df[f'rsi_{feature}_{lb}'] =  up / (up + np.abs(down) + 1) 
    return rsi_df      
    
def diff_level_prices(df, levels, instruments):
    features = product(levels, instruments)
    diff_all = pd.DataFrame()
    for lvl, inst in features:
        diff_all[f'{inst}_diff_level_{lvl}'] = (df[f'{inst}_ask_price_{lvl}'] - df[f'{inst}_bid_price_{lvl}'])
    return diff_all

def diff_level_volumes(df, levels, instruments):
    features = product(levels, instruments)
    diff_all = pd.DataFrame()
    for lvl, inst in features:
        diff_all[f'{inst}_diff_vol_{lvl}'] = (df[f'{inst}_ask_vol_{lvl}'] - df[f'{inst}_bid_vol_{lvl}'])
    return diff_all

def diff_shifts_features(df, features_1: List[str], features_2: List[str], periods: List[Tuple[int, int]]):
    diff_features_df = pd.DataFrame()
    for per_1, per_2 in periods:
        for feat_1_name, feat_2_name in zip(features_1, features_2):
            diff_features_df[f'{feat_1_name}_{per_1}_diff_{feat_2_name}_{per_2}'] = (df[feat_1_name].shift(periods=per_1) - df[feat_2_name].shift(periods=per_2))
    return diff_features_df

def diff_midprices(df, periods, instruments):
    diff_mdp_inst = pd.DataFrame()
    for inst in instruments:
        midprice = (df.loc[:, f'{inst}_ask_price_0'] + df.loc[:, f'{inst}_bid_price_0']) / 2
        for period in periods:
            diff_mdp_inst[f'{inst}_diff_{period}'] = midprice.diff(periods=period)
    return diff_mdp_inst

def ma_diffs_calc(df, features_1, features_2=None, periods: List[Tuple[int, int]]= None):
    if features_2 is None:
        features_2 = features_1
        
    ma_diff_features_df = pd.DataFrame()
    for per_1, per_2 in periods:
        for feat_1_name, feat_2_name in zip(features_1, features_2):
            ma_diff_features_df[f'ma_{feat_1_name}_{per_1}_diff_{feat_2_name}_{per_2}'] = (df[feat_1_name].rolling(min_periods=1, window=per_1).mean() - df[feat_2_name].rolling(min_periods=1, window=per_2).mean())
    return ma_diff_features_df

def squared_diffs_calc(df, features, lookbacks):
    df_volatility = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            df_volatility[f'volat_{feature}_{lb}'] = ((df[feature] - df[feature].shift(periods=1))**2).rolling(min_periods=1, window=lb).mean()
    return df_volatility

def diff_twap_feature(df, features, lookbacks):
    assert 'time_1_tick_diff' in df.columns
    
    diff_twap_inst = pd.DataFrame()
    for feat in features:
        for lb in lookbacks:
            diff_twap_inst[f'twap_diff_{feat}_{lb}'] = (df[feat].diff() * df['time_1_tick_diff']).rolling(min_periods=1, window=lb).mean()
    return diff_twap_inst

def zscore_calc(df, features, lookbacks: List[int]):
    df_zscore = pd.DataFrame()
    for feature in features:
        for lb in lookbacks:
            df_zscore[f'zscore_{feature}_{lb}'] = (df[feature] - df[feature].rolling(min_periods=lb // 2, window=lb).mean()) / df[feature].rolling(min_periods=lb // 2, window=lb).std()
    return df_zscore

def corr_features_calc(df, features_1, features_2, lookbacks: List[int]):
    df_corr = pd.DataFrame()
    for feature_1, feature_2 in zip(features_1, features_2):
        for lb in lookbacks:
            df_corr[f'corr_{feature_1}_{feature_2}_{lb}'] = df[feature_1].rolling(min_periods=lb // 2, window=lb).corr(df[feature_2])
    return df_corr


def ret_spot_perp(df, spot, perp):
    ret_spot_perp_df = pd.DataFrame()
    ret_spot_perp_df[f'ret_{spot}_{perp}'] = (df[f'{perp}_midprice_0'] - df[f'{spot}_midprice_0']) / df[f'{spot}_midprice_0']
    return ret_spot_perp_df

def time_diff_mili(df, time_col_name: str):
    time_diff = pd.DataFrame()
    time_diff['time_diff_mili'] = df[time_col_name].diff() / 10**6
    return time_diff

def time_feature_move(df, feature_name, index_name: str, cnt_move_to_check=20, move_unit=0):
    @njit
    def calc(vals, time_milli, cnt_move_to_check):
        val_len = len(vals)
        time_for_move = np.empty(val_len)
        time_for_move.fill(np.nan)
        
        for ind in range(val_len - 1, -1, -1):
            cur_move_cnt = 0
            cur_move_check_ind = ind
            cur_tfm = 0
            while cur_move_cnt <= cnt_move_to_check:
                if cur_move_check_ind <= 1:
                    break
                    
                is_move = (abs(vals[cur_move_check_ind] - vals[cur_move_check_ind-1]) >= move_unit)
                
                if is_move:
                    cur_move_cnt += 1
                
                cur_move_check_ind -= 1
                    
            else:
                time_for_move[ind] = (time_milli[ind] - time_milli[cur_move_check_ind-1]) / cnt_move_to_check 
                

        return time_for_move
    
    vals = df[feature_name].values
    time_milli = (df[index_name] // 10**6).values
    
    
    time_move = calc(vals, time_milli, cnt_move_to_check)
    return pd.DataFrame(time_move, columns=[f'ttm_{cnt_move_to_check}_{feature_name}'], index=df[index_name])

def diff_side_vwaps_up_level(df, levels, instruments):
    features = product(levels, instruments)
    diff_vwaps = pd.DataFrame()
    for lvl, inst in features:
        diff_vwaps[f'{inst}_diff_vwaps_{lvl}'] = vwap_side(df, up_level=lvl, instrument=inst, side='ask').iloc[:,0] - vwap_side(df, up_level=lvl, instrument=inst, side='bid').iloc[:,0]
    return diff_vwaps

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ boyko's features
def OFi090_calc(df: pd.DataFrame, inst, time_intervals_ns: List[int], a_depth=6, b_depth=5):
    OFi090_df = pd.DataFrame()
    a090 = vwap_all(df, instrument=inst, up_level=1)
    bdi090a = bookdepth(df, inst=inst, side='ask', qty_depths=[a_depth]).iloc[1:, 0]
    bdi090b = bookdepth(df, inst=inst, side='bid', qty_depths=[b_depth]).iloc[1:, 0]
    
    bki090a = df[f'{inst}_ask_vol_{0}']
    bki090b = df[f'{inst}_bid_vol_{0}']
    
    bid_sign = np.diff(bki090b) > 0
    OFi090_df[f'OFi090_{inst}_ad{a_depth}_bd{b_depth}'] =  bid_sign * bdi090b - (1 - bid_sign) * bdi090b.shift(1)

    ask_sign = np.diff(bki090a) > 0
    OFi090_df[f'OFi090_{inst}_ad{a_depth}_bd{b_depth}'] += ask_sign * bdi090a - (1 - ask_sign) * bdi090a.shift(1)
    
    of_val = OFi090_df.iloc[:,0].values
    of_times = OFi090_df.index.values
    
    for time_ns in time_intervals_ns:
        col_name = f'OFI{int(time_ns / 1e3)}090_{inst}_ad{a_depth}_bd{b_depth}'
        OFi090_df[col_name] = eboxsum(of_val, of_times, time_ns) / (bki090a.values[1:] + bki090b.values[1:])
        
    return OFi090_df
    
    
def log3(df, up_levels: List[int], price_limit_lot: List[Optional[int]], inst):
    log3_df = pd.DataFrame()
    
    for uplvl, plimit in zip(up_levels, price_limit_lot):
        if plimit:
            best_ask = df[f'{inst}_ask_vol_0']
            best_bid = df[f'{inst}_bid_vol_0']    
            cum_ask_vol = sum([df[f'{inst}_ask_vol_{lvl}'] * (plimit >= df[f'{inst}_ask_vol_{lvl}'] - best_ask) for lvl in range(uplvl)])
            cum_bid_vol = sum([df[f'{inst}_bid_vol_{lvl}'] * (plimit >= best_bid - df[f'{inst}_bid_vol_{lvl}']) for lvl in range(uplvl)])
        else:
            cum_ask_vol = sum([df[f'{inst}_ask_vol_{lvl}'] for lvl in range(uplvl)])
            cum_bid_vol = sum([df[f'{inst}_bid_vol_{lvl}'] for lvl in range(uplvl)])
            
        log3_df[f'{inst}_log3_{uplvl}'] = np.power(np.log(cum_ask_vol / cum_bid_vol), 3)
        
    return log3_df


def bookdepth(df, inst, qty_depths: List[float], side, num_levels=6):
    assert side in ('ask', 'bid')
    
    bd_df = pd.DataFrame()
    side_sign = 1 if side == 'bid' else -1
    
    for qty in qty_depths:
        depth_price = df[f'{inst}_{side}_price_0'] - side_sign * qty
        bd_df[f'{inst}_bookdepth_{side}_{qty}'] = pd.Series(np.zeros(df.shape[0]), index=df.index)
        for lvl in range(num_levels):
            mask_lvl = side_sign * (df[f'{inst}_{side}_price_{lvl}'] - depth_price) > 0
            bd_df[f'{inst}_bookdepth_{side}_{qty}'] += df[f'{inst}_{side}_vol_{lvl}'] * mask_lvl
            
    return bd_df


def Lvs_feature(df_lobs, trades_df, anvl_flgs: List, spread_vals):
    anvl_w_ns =  30 * SECOND
    predict_times = df_lobs['received_ts'].values
    annvol_df_lst = ANNVOL_df(trades_df,
                          times_on_ns=predict_times, 
                          flags=anvl_flgs,
                          window_ns=anvl_w_ns,
                          spread_vals=spread_vals)
    
    feature_param_map =  {'wdelta': 
                               [
                                {'ns_period': 1 * SECOND,
                                'feature_df_col_name': 'L1090'}, 

                               {'ns_period': 15 * SECOND,
                                'feature_df_col_name': 'L15090'},

                               {'ns_period': 30 * SECOND,
                                'feature_df_col_name': 'L30090'},
                               ]}
    Lvs_features_lst = []
    for flag, df in zip(anvl_flgs, annvol_df_lst):
        feature_param_map_ = deepcopy(feature_param_map)
        for i in range(3):
            feature_param_map_['wdelta'][i]['feature_df_col_name'] += flag
        df = np.log10(pd.concat(feature_scheduler(df=df, feature_param_map=feature_param_map_),axis=1)).fillna(method='ffill')
        Lvs_features_lst.append(df)
        
    return Lvs_features_lst

def twRs_feature(df_lobs, ext_window_ns: int, feat_windows_ns: List[int], fname=None):
    cfg = {
    'wmax': [
            {
             'ns_period': 60 * SECOND,
             'feature_df_col_name': 'max60a090',
             'feature_col_name': fname
            }
            ], 

    'wmin': [
            {
            'ns_period': 60 * SECOND, 
            'feature_df_col_name': 'min60a090',
            'feature_col_name': fname
            }
            ]
    }
    
    df_feature = pd.DataFrame()
    if fname is None:
        feature_to_use = df_lobs.iloc[:,0]
    else:
        feature_to_use = df_lobs[fname]

    wminmax = pd.concat(feature_scheduler(df=df_lobs, feature_param_map=cfg), axis=1)
    cfg = {'TWA': []}
    for fw in feat_windows_ns:
        cfg['TWA'].append({'ns_period': fw, 'feature_df_col_name': f'{fw}'})
        
    tmax = feature_scheduler(df=wminmax['max60a090'], feature_param_map=cfg)
    tmin = feature_scheduler(df=wminmax['min60a090'], feature_param_map=cfg)
    # print(tmax)
    # print(tmin)
    # print(feature_to_use - tmax[0])
    # return feature_to_use, tmax[0]
    # print(feature_to_use)
    # print(tmax[0])
    # 1/0
    for fw_ind, fw in enumerate(feat_windows_ns):
        df_feature[f'twR_{fw / SECOND}sec'] = np.minimum(feature_to_use - tmin[fw_ind].iloc[:,0], 0) + np.maximum(feature_to_use - tmax[fw_ind].iloc[:,0], 0)
    return df_feature
    
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ 