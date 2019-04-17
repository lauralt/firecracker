import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
import json

iops_list_libaio = []
bw_list_libaio = []
iops_list_mmap = []
bw_list_mmap = []
iops_list_sync = []
bw_list_sync = []
bs_list = [4, 8, 16, 32]
write_mmap_dict = {'bw': bw_list_mmap, 'iops': iops_list_mmap}
write_libaio_dict = {'bw': bw_list_libaio, 'iops': iops_list_libaio}
write_sync_dict = {'bw': bw_list_sync, 'iops': iops_list_sync}

fio_results_file = 'fio_results_iod1.txt'
f = open(fio_results_file, 'r')
jobs = f.readlines()
for job in jobs:
    if job.startswith("{'fio version': 'fio-3.1'"):
        json_string_job = job.replace("'", "\"")
        fio_results = json.loads(json_string_job)
        io_engine = fio_results["global options"]["ioengine"]
        block_size = fio_results["global options"]["bs"]
        job_results = fio_results["jobs"]
        write_stats = job_results[0]["write"]
        if job_results[0]["job options"]["rw"] == "randwrite":
            for fio_param in write_mmap_dict:
                if fio_param in write_stats:
                    if io_engine == "mmap":
                        write_mmap_dict.get(fio_param).append(write_stats[fio_param])
                    elif io_engine == "libaio":
                        write_libaio_dict.get(fio_param).append(write_stats[fio_param])
                    else:
                        write_sync_dict.get(fio_param).append(write_stats[fio_param])

plt.figure()
grid = gridspec.GridSpec(1, 2)
fig1 = plt.subplot(grid[0, 0])
plt.plot(bs_list, write_mmap_dict.get('bw')[0:4], marker="o", label='mmap')
plt.plot(bs_list, write_libaio_dict.get('bw')[0:4], marker="o", label='libaio')
plt.plot(bs_list, write_sync_dict.get('bw')[0:4], marker="o", label='sync')
plt.title('Fio results for random writes for different block sizes', loc='center')
plt.xlabel('Block sizes [kB]')
plt.ylabel('Bandwidth [kB/sec]')
plt.legend()

fig2 = plt.subplot(grid[0, 1])
plt.plot(bs_list, write_mmap_dict.get('iops')[0:4], marker="o", label='mmap')
plt.plot(bs_list, write_libaio_dict.get('iops')[0:4], marker="o", label='libaio')
plt.plot(bs_list, write_sync_dict.get('iops')[0:4], marker="o", label='sync')
plt.title('Fio results for random writes for different block sizes', loc='center')
plt.xlabel('Block sizes [kB]')
plt.ylabel('IOPS')
plt.legend()

# fig3 = plt.subplot(grid[1, 0])
# plt.plot(bs_list, write_mmap_dict.get('bw')[4:8], marker="o", label='mmap')
# plt.plot(bs_list, write_libaio_dict.get('bw')[4:8], marker="o", label='libaio')
# plt.plot(bs_list, write_sync_dict.get('bw')[4:8], marker="o", label='sync')
# plt.title('Fio results for random writes for different block sizes (vsock)', loc='center')
# plt.xlabel('Block sizes [kB]')
# plt.ylabel('Bandwidth [kB/sec]')
# plt.legend()
#
# fig4 = plt.subplot(grid[1, 1])
# plt.plot(bs_list, write_mmap_dict.get('iops')[4:8], marker="o", label='mmap')
# plt.plot(bs_list, write_libaio_dict.get('iops')[4:8], marker="o", label='libaio')
# plt.plot(bs_list, write_sync_dict.get('iops')[4:8], marker="o", label='sync')
# plt.title('Fio results for random writes for different block sizes (vsock)', loc='center')
# plt.xlabel('Block sizes [kB]')
# plt.ylabel('IOPS')
# plt.legend()

plt.show()
