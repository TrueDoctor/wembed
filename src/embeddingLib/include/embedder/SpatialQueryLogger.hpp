#pragma once

#ifdef ENABLE_SPATIAL_LOGGING
#include <fstream>
#include <vector>
#include "DVec.hpp"

namespace spatial_logging {
    void init_logging(const std::string& prefix);
    void close_logging();
    void log_iteration(int iter);
    void log_positions(const std::vector<std::vector<double>>& positions, const std::vector<double>& weights);
    void log_query_nearest(CVecRef point, unsigned int k);
    void log_query_sphere(CVecRef min_corner, CVecRef max_corner, CVecRef point, double radius); 
    void log_query_box(CVecRef min_corner, CVecRef max_corner);
}

#define LOG_SPATIAL_INIT(prefix) spatial_logging::init_logging(prefix)
#define LOG_SPATIAL_CLOSE() spatial_logging::close_logging()
#define LOG_SPATIAL_QUERY_NEAREST(point, k) spatial_logging::log_query_nearest(point, k)
#define LOG_SPATIAL_QUERY_SPHERE(min, max, point, r) spatial_logging::log_query_sphere(min, max, point, r)
#define LOG_SPATIAL_QUERY_BOX(min, max) spatial_logging::log_query_box(min, max)
#define LOG_SPATIAL_POSITIONS(pos, weights) spatial_logging::log_positions(pos, weights)
#define LOG_SPATIAL_ITERATION(iter) spatial_logging::log_iteration(iter)

#else

#define LOG_SPATIAL_INIT(prefix)
#define LOG_SPATIAL_CLOSE()
#define LOG_SPATIAL_QUERY_NEAREST(point, k)
#define LOG_SPATIAL_QUERY_SPHERE(min, max, point, r)
#define LOG_SPATIAL_QUERY_BOX(min, max)
#define LOG_SPATIAL_POSITIONS(pos, weight)
#define LOG_SPATIAL_ITERATION(iter)

#endif
