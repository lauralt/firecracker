import json
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
import numpy as np


def plot_graph(metric, io_pattern, jobs):
    iod_1_list = []
    iod_2_list = []
    iod_8_list = []
    iod_32_list = []
    bs_list = ['4', '32', '128', '512', '1024']
    metric_dict = {'iod_1': iod_1_list, 'iod_2': iod_2_list, 'iod_8': iod_8_list, 'iod_32': iod_32_list}
    units_dict = {'iops': 'operatii I/O/sec', 'bw': 'kB/sec'}
    iodepth_list = ['iod_1', 'iod_2', 'iod_8', 'iod_32']
    read_op = {'randread', 'read'}
    ind = np.arange(len(bs_list))
    bar_algn = -0.3
    if io_pattern in read_op:
        io_op = 'read'
    else:
        io_op = 'write'
    for job in jobs:
        if job.startswith("{'fio version': 'fio-3.1'"):
            json_string_job = job.replace("'", "\"")
            fio_results = json.loads(json_string_job)
            job_results = fio_results['jobs']
            stats = job_results[0][io_op]
            io_depth = job_results[0]['job options']['iodepth']
            io_depth_str = 'iod_' + str(io_depth)
            if io_depth_str in metric_dict:
                if job_results[0]['job options']['rw'] == io_pattern:
                    if metric in stats:
                        metric_dict[io_depth_str].append(stats[metric])

    for io_depth in iodepth_list:
        plt.bar(ind + bar_algn, metric_dict.get(io_depth)[0:5], width=0.2, align='center',
                label='iodepth=' + io_depth.split("_")[1])
        bar_algn += 0.2
    plt.xticks(ind, bs_list)
    plt.title('Fio results for ' + io_pattern + ' operations for different block sizes', loc='center')
    plt.xlabel('Block size [kB]')
    plt.ylabel(metric.upper() + " [" + units_dict.get(metric) + "]")
    plt.legend()


io_pattern_list = ['randread', 'randwrite', 'read', 'write']
fio_metrics = ['iops', 'bw']
fio_results_file = 'fio_results_file.txt'
f = open(fio_results_file, 'r')
fio_jobs = f.readlines()
for pattern in io_pattern_list:
    plt.figure()
    grid = gridspec.GridSpec(1, 2)
    i = 0
    for fio_metric in fio_metrics:
        plt.subplot(grid[0, i])
        plot_graph(fio_metric, pattern, fio_jobs)
        i += 1
plt.show()
