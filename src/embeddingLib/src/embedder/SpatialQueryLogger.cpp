// SpatialQueryLogger.cpp
#ifndef ENABLE_SPATIAL_LOGGING
#define ENABLE_SPATIAL_LOGGING
#endif
#ifdef ENABLE_SPATIAL_LOGGING

#include "SpatialQueryLogger.hpp"
#include <iostream>
#include "EmbedderOptions.hpp"

namespace spatial_logging {
    static std::ofstream query_log;
    static std::ofstream position_log;
    static EmbedderOptions options;
    static int current_iteration = 0;
    
    void init_logging(EmbedderOptions options) {
        std::cout << "Log Mod after init: " << options.iteration_logging_mod << std::endl;

        std::string query_filename = options.loggingOutput + "_queries.log";
        std::string position_filename = options.loggingOutput + "_positions.log";
        spatial_logging::options = options;  // Store the options for later use

        std::cout << "Log Mod after storing options: " << spatial_logging::options.iteration_logging_mod << std::endl;
        
        query_log.open(query_filename);
        position_log.open(position_filename);
        std::cout << "saving queries to: " << query_filename << std::endl;
        
        if (!query_log.is_open() || !position_log.is_open()) {
            std::cerr << "Failed to open log files with prefix: " << options.loggingOutput << std::endl;
            return;
        }
    }

    void close_logging() {
        if (query_log.is_open()) query_log.close();
        if (position_log.is_open()) position_log.close();
    }

    void log_iteration(int iter) {
        current_iteration = iter;
        if (iter % spatial_logging::options.iteration_logging_mod != 0) return;  // Only log every nth iteration

        std::cout << "Logging iteration " << iter << std::endl;

        std::cout << "Iter % " << options.iteration_logging_mod << " == 0" << std::endl;

        if (query_log.is_open()) {
            query_log << "ITERATION " << iter << "\n";
            query_log.flush();
        }
        if (position_log.is_open()) {
            position_log << "ITERATION " << iter << "\n";
            position_log.flush();
        }
    }

    void log_positions(const std::vector<std::vector<double>>& positions, const std::vector<double>& weights) {
        if (current_iteration % spatial_logging::options.iteration_logging_mod != 0) return;  // Only log every nth iteration

        if (!position_log.is_open()) return;
        // std::cout << "logging positions " << std::endl;
        
        position_log << positions.size() << " " << positions[0].size() << "\n";
        for (size_t i = 0; i < positions.size(); ++i) {
            position_log << i << " " << weights[i] << " " << positions[i].size() << " ";
            for (double x : positions[i]) {
                position_log << x << " ";
            }
            position_log << "\n";
        }
        position_log << "---\n";
        position_log.flush();

        if (!query_log.is_open()) return;

        query_log << "---\n";
    }

    void log_query_nearest(CVecRef point, unsigned int k) {
        if (current_iteration % spatial_logging::options.iteration_logging_mod != 0) return;  // Only log every nth iteration

        std::cout << "nearest " << std::endl;
        if (!query_log.is_open()) return;
        
        query_log << "NEAREST " << k << " " << point.dimension() << " ";
        for (int i = 0; i < point.dimension(); i++) {
            query_log << point[i] << " ";
        }
        query_log << "\n";
        query_log.flush();
    }

    void log_query_sphere(CVecRef min_corner, CVecRef max_corner, CVecRef point, double radius) {
        if (current_iteration % spatial_logging::options.iteration_logging_mod != 0) return;  // Only log every nth iteration

        if (!query_log.is_open()) return;
        
        query_log << "SPHERE " << radius << " " << point.dimension() << " ";
        for (int i = 0; i < point.dimension(); i++) {
            query_log << min_corner[i] << " " << max_corner[i] << " " << point[i] << " ";
        }
        query_log << "\n";
        query_log.flush();
    }

    void log_query_box(CVecRef min_corner, CVecRef max_corner) {
        if (current_iteration % spatial_logging::options.iteration_logging_mod != 0) return;  // Only log every nth iteration

        std::cout << "box " << std::endl;
        if (!query_log.is_open()) return;
        
        query_log << "BOX " << min_corner.dimension() << " ";
        for (int i = 0; i < min_corner.dimension(); i++) {
            query_log << min_corner[i] << " " << max_corner[i] << " ";
        }
        query_log << "\n";
        query_log.flush();
    }
}
#endif
