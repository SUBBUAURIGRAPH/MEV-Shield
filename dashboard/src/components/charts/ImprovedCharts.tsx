import React, { useRef, useEffect } from 'react';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from 'chart.js';
import { Line, Bar, Doughnut } from 'react-chartjs-2';
import { Box, Paper, Typography } from '@mui/material';
import { 
  getLineChartOptions, 
  getBarChartOptions, 
  getDoughnutChartOptions,
  chartColors,
  createGradient 
} from './ChartConfigs';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

interface ChartProps {
  title?: string;
  height?: number | string;
}

// MEV Activity Chart with improved scaling
export const MEVActivityChart: React.FC<ChartProps> = ({ title = "MEV Activity", height = 350 }) => {
  const chartRef = useRef<any>(null);

  const data = {
    labels: ['00:00', '04:00', '08:00', '12:00', '16:00', '20:00', '24:00'],
    datasets: [
      {
        label: 'Protected Transactions',
        data: [120, 190, 300, 450, 420, 380, 410],
        borderColor: chartColors.primary,
        backgroundColor: (context: any) => {
          const ctx = context.chart.ctx;
          return createGradient(ctx, chartColors.gradients.green[0], chartColors.gradients.green[1]);
        },
        fill: true,
        tension: 0.4,
      },
      {
        label: 'MEV Attacks Blocked',
        data: [30, 45, 65, 80, 75, 70, 78],
        borderColor: chartColors.error,
        backgroundColor: 'rgba(244, 67, 54, 0.1)',
        borderDash: [5, 5],
      },
    ],
  };

  return (
    <Paper elevation={2} sx={{ p: 3, height: height, borderRadius: 2 }}>
      <Typography variant="h6" gutterBottom sx={{ mb: 2, fontWeight: 600 }}>
        {title}
      </Typography>
      <Box sx={{ height: 'calc(100% - 40px)' }}>
        <Line ref={chartRef} data={data} options={getLineChartOptions()} />
      </Box>
    </Paper>
  );
};

// Value Protected Chart with ETH formatting
export const ValueProtectedChart: React.FC<ChartProps> = ({ title = "Value Protected (ETH)", height = 350 }) => {
  const data = {
    labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
    datasets: [
      {
        label: 'ETH Protected',
        data: [12.5, 19.3, 15.8, 22.1, 18.6, 25.4, 20.2],
        backgroundColor: chartColors.gradients.blue[0],
        borderColor: chartColors.secondary,
        borderWidth: 2,
        borderRadius: 8,
        borderSkipped: false,
      },
    ],
  };

  return (
    <Paper elevation={2} sx={{ p: 3, height: height, borderRadius: 2 }}>
      <Typography variant="h6" gutterBottom sx={{ mb: 2, fontWeight: 600 }}>
        {title}
      </Typography>
      <Box sx={{ height: 'calc(100% - 40px)' }}>
        <Bar data={data} options={getBarChartOptions(title)} />
      </Box>
    </Paper>
  );
};

// MEV Type Distribution Chart
export const MEVDistributionChart: React.FC<ChartProps> = ({ title = "MEV Attack Types", height = 350 }) => {
  const data = {
    labels: ['Sandwich', 'Front-run', 'Back-run', 'Arbitrage', 'Liquidation'],
    datasets: [
      {
        data: [35, 25, 20, 15, 5],
        backgroundColor: [
          chartColors.error,
          chartColors.warning,
          chartColors.info,
          chartColors.purple,
          chartColors.indigo,
        ],
        borderWidth: 0,
      },
    ],
  };

  return (
    <Paper elevation={2} sx={{ p: 3, height: height, borderRadius: 2 }}>
      <Typography variant="h6" gutterBottom sx={{ mb: 2, fontWeight: 600 }}>
        {title}
      </Typography>
      <Box sx={{ height: 'calc(100% - 40px)' }}>
        <Doughnut data={data} options={getDoughnutChartOptions()} />
      </Box>
    </Paper>
  );
};

// Network Performance Chart
export const NetworkPerformanceChart: React.FC<ChartProps> = ({ title = "Network Performance", height = 350 }) => {
  const data = {
    labels: ['1h', '2h', '3h', '4h', '5h', '6h'],
    datasets: [
      {
        label: 'Transaction Throughput',
        data: [850, 920, 880, 950, 1020, 980],
        borderColor: chartColors.success,
        backgroundColor: 'rgba(76, 175, 80, 0.1)',
        yAxisID: 'y',
      },
      {
        label: 'Protection Latency (ms)',
        data: [45, 42, 48, 41, 39, 43],
        borderColor: chartColors.warning,
        backgroundColor: 'rgba(255, 152, 0, 0.1)',
        yAxisID: 'y1',
      },
    ],
  };

  const options: any = {
    ...getLineChartOptions(),
    scales: {
      x: {
        grid: { display: false },
      },
      y: {
        type: 'linear' as const,
        display: true,
        position: 'left' as const,
        beginAtZero: true,
        title: {
          display: true,
          text: 'Transactions/min',
        },
      },
      y1: {
        type: 'linear' as const,
        display: true,
        position: 'right' as const,
        beginAtZero: true,
        grid: { drawOnChartArea: false },
        title: {
          display: true,
          text: 'Latency (ms)',
        },
      },
    },
  };

  return (
    <Paper elevation={2} sx={{ p: 3, height: height, borderRadius: 2 }}>
      <Typography variant="h6" gutterBottom sx={{ mb: 2, fontWeight: 600 }}>
        {title}
      </Typography>
      <Box sx={{ height: 'calc(100% - 40px)' }}>
        <Line data={data} options={options} />
      </Box>
    </Paper>
  );
};

// Exchange Integration Volume Chart
export const ExchangeVolumeChart: React.FC<ChartProps> = ({ title = "Exchange Integration Volume", height = 350 }) => {
  const data = {
    labels: ['Binance', 'Coinbase', 'Kraken', 'OKX', 'Bybit', 'Gate.io'],
    datasets: [
      {
        label: 'Daily Volume (USD)',
        data: [4500000, 3200000, 2100000, 1800000, 1500000, 900000],
        backgroundColor: [
          'rgba(76, 175, 80, 0.7)',
          'rgba(33, 150, 243, 0.7)',
          'rgba(156, 39, 176, 0.7)',
          'rgba(255, 152, 0, 0.7)',
          'rgba(244, 67, 54, 0.7)',
          'rgba(0, 188, 212, 0.7)',
        ],
        borderColor: [
          chartColors.success,
          chartColors.secondary,
          chartColors.purple,
          chartColors.warning,
          chartColors.error,
          chartColors.info,
        ],
        borderWidth: 2,
      },
    ],
  };

  const options: any = {
    ...getBarChartOptions(),
    indexAxis: 'y' as const,
    scales: {
      x: {
        beginAtZero: true,
        ticks: {
          callback: function(value: any) {
            return '$' + (value / 1000000).toFixed(1) + 'M';
          },
        },
      },
    },
  };

  return (
    <Paper elevation={2} sx={{ p: 3, height: height, borderRadius: 2 }}>
      <Typography variant="h6" gutterBottom sx={{ mb: 2, fontWeight: 600 }}>
        {title}
      </Typography>
      <Box sx={{ height: 'calc(100% - 40px)' }}>
        <Bar data={data} options={options} />
      </Box>
    </Paper>
  );
};